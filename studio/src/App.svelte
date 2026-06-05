<script lang="ts">
  import { onMount } from 'svelte'
  import { listen } from '@tauri-apps/api/event'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { message, open } from '@tauri-apps/plugin-dialog'
  import { revealItemInDir } from '@tauri-apps/plugin-opener'
  import AssetRenameModal from './components/AssetRenameModal.svelte'
  import FileTree from './components/FileTree.svelte'
  import JsonEditor from './components/JsonEditor.svelte'
  import SiteFormEditor from './components/SiteFormEditor.svelte'
  import PageFormEditor from './components/PageFormEditor.svelte'
  import ImagePreviewPanel from './components/ImagePreviewPanel.svelte'
  import PreviewPanel from './components/PreviewPanel.svelte'
  import ProblemsPanel from './components/ProblemsPanel.svelte'
  import ResizableWorkspace from './components/ResizableWorkspace.svelte'
  import LogPanel from './components/LogPanel.svelte'
  import StudioBundleProvider from './components/StudioBundleProvider.svelte'
  import {
    buildSite,
    createSiteFromTemplate,
    getStudioSettings,
    importBundleAsset,
    renameBundleAsset,
    listBundleFiles,
    listContentBundles,
    projectInfoForRoot,
    readBundleFile,
    resolveProjectRoot,
    saveStudioSettings,
    startPreviewServer,
    validateSite,
    writeBundleFile,
    type BuildSiteResult,
    type BundleFileEntry,
    type BuiltinThemeId,
    type Diagnostic,
    type ProjectRootInfo,
    type ValidateSiteResult,
  } from './lib/studio-api'
  import {
    applyBuiltinTheme,
    BUILTIN_THEME_IDS,
    DEFAULT_BUILTIN_THEME,
    getBuiltinThemeLabel,
    isBuiltinThemeId,
    migrateBuiltinThemeId,
  } from './lib/styles/themeTokens'
  import {
    DEFAULT_SIDEBAR_PX,
    type WorkspaceLayout,
  } from './lib/workspace-layout'
  import {
    isPageJson,
    isSiteJson,
    supportsFormView,
  } from './lib/editor-schema'
  import { isImageFile } from './lib/image-files'
  import {
    copyAssetPathToClipboard,
    IMAGE_DIALOG_FILTERS,
    removeAssetWithConfirm,
  } from './lib/asset-actions'

  const DEFAULT_SITE = 'content/kometa'
  const PREVIEW_PORT = 8080
  const DEFAULT_LOG_HEIGHT_PX = 112

  type SiteEditorView = 'json' | 'form'

  type EditorTab = {
    relativePath: string
    content: string
    savedContent: string
    siteView?: SiteEditorView
  }

  function siteViewForTab(tab: EditorTab): SiteEditorView {
    return tab.siteView ?? (supportsFormView(tab.relativePath) ? 'form' : 'json')
  }

  function setSiteView(relativePath: string, siteView: SiteEditorView) {
    tabs = tabs.map((t) =>
      t.relativePath === relativePath ? { ...t, siteView } : t,
    )
  }

  let projectInfo = $state<ProjectRootInfo | null>(null)
  let bundles = $state<string[]>([])
  let selectedSite = $state(DEFAULT_SITE)
  let strict = $state(false)
  let fileEntries = $state<BundleFileEntry[]>([])
  let tabs = $state<EditorTab[]>([])
  let activeTabPath = $state<string | null>(null)
  let imagePreviewPath = $state<string | null>(null)
  let problems = $state<Diagnostic[]>([])
  let logs = $state<string[]>([])
  let lastOutputDir = $state<string | null>(null)
  let previewUrl = $state<string | null>(null)
  let previewRefreshKey = $state(0)
  let previewGeneration = $state(0)
  let busy = $state(false)
  let themeId = $state<BuiltinThemeId>(DEFAULT_BUILTIN_THEME)
  let sidebarWidth = $state(DEFAULT_SIDEBAR_PX)
  let previewWidth = $state(360)
  let logCollapsed = $state(true)
  let logHeightPx = $state(DEFAULT_LOG_HEIGHT_PX)
  let savedWorkspaceLayout = $state<WorkspaceLayout | null>(null)
  let renameAssetPath = $state<string | null>(null)

  const imageAssets = $derived(
    fileEntries
      .filter((e) => !e.is_dir && isImageFile(e.relative_path))
      .map((e) => e.relative_path),
  )

  function workspaceLayoutPayload(): WorkspaceLayout {
    return {
      sidebar_px: sidebarWidth,
      preview_px: previewWidth,
      log_collapsed: logCollapsed,
      log_height_px: logHeightPx,
    }
  }

  function appendLog(line: string) {
    const stamp = new Date().toLocaleTimeString()
    logs = [...logs, `[${stamp}] ${line}`].slice(-200)
  }

  function activeTab(): EditorTab | undefined {
    return tabs.find((t) => t.relativePath === activeTabPath)
  }

  function isDirty(tab: EditorTab) {
    return tab.content !== tab.savedContent
  }

  function setProblemsFromResult(result: ValidateSiteResult | BuildSiteResult) {
    problems = [...result.warnings, ...result.errors]
  }

  async function persistStudioSettings(overrides: {
    last_project_root?: string | null
    last_selected_site?: string | null
    theme?: BuiltinThemeId
    workspace_layout?: WorkspaceLayout
  } = {}) {
    await saveStudioSettings({
      last_project_root:
        overrides.last_project_root !== undefined
          ? overrides.last_project_root
          : projectInfo?.project_root,
      last_selected_site:
        overrides.last_selected_site !== undefined
          ? overrides.last_selected_site
          : selectedSite,
      theme: overrides.theme ?? themeId,
      workspace_layout: overrides.workspace_layout ?? workspaceLayoutPayload(),
    })
  }

  function applySiteSelection(savedSite: string | null | undefined) {
    if (savedSite && bundles.includes(savedSite)) {
      selectedSite = savedSite
    } else if (bundles.includes(DEFAULT_SITE)) {
      selectedSite = DEFAULT_SITE
    } else if (bundles.length > 0) {
      selectedSite = bundles[0]
    }
  }

  function onWorkspaceLayoutCommit() {
    void persistStudioSettings()
  }

  function toggleLogPanel() {
    logCollapsed = !logCollapsed
    void persistStudioSettings()
  }

  function onLogHeightChange(heightPx: number) {
    logHeightPx = heightPx
    void persistStudioSettings()
  }

  async function persistProjectRoot(root: string) {
    await persistStudioSettings({ last_project_root: root })
  }

  function setTheme(id: BuiltinThemeId) {
    themeId = id
    applyBuiltinTheme(id, document.documentElement)
    void persistStudioSettings({ theme: id })
  }

  async function loadProject(root: string, savedSite?: string | null) {
    projectInfo = await projectInfoForRoot(root)
    await persistProjectRoot(projectInfo.project_root)
    bundles = await listContentBundles(projectInfo.project_root)
    applySiteSelection(savedSite)
    await refreshFileTree()
    appendLog(`Opened project: ${projectInfo.project_root}`)
  }

  async function refreshFileTree() {
    if (!projectInfo) return
    fileEntries = await listBundleFiles(projectInfo.project_root, selectedSite)
  }

  async function onOpenProject() {
    try {
      const picked = await open({
        directory: true,
        multiple: false,
        title: 'Open project folder',
      })
      if (!picked || typeof picked !== 'string') return
      busy = true
      const settings = await getStudioSettings()
      await loadProject(picked, settings.last_selected_site)
    } catch (err: unknown) {
      appendLog(`Open project failed: ${String(err)}`)
    } finally {
      busy = false
    }
  }

  function selectedTreePath(): string | null {
    return imagePreviewPath ?? activeTabPath
  }

  async function onSiteChange() {
    previewGeneration += 1
    tabs = []
    activeTabPath = null
    imagePreviewPath = null
    problems = []
    previewUrl = null
    lastOutputDir = null
    previewRefreshKey += 1
    void persistStudioSettings({ last_selected_site: selectedSite })
    await refreshFileTree()
  }

  async function openImage(relativePath: string) {
    if (!projectInfo || !isImageFile(relativePath)) return
    imagePreviewPath = relativePath
    activeTabPath = null
  }

  async function openFile(relativePath: string) {
    if (!projectInfo) return
    imagePreviewPath = null
    const existing = tabs.find((t) => t.relativePath === relativePath)
    if (existing) {
      activeTabPath = relativePath
      return
    }
    try {
      const content = await readBundleFile(
        projectInfo.project_root,
        selectedSite,
        relativePath,
      )
      tabs = [...tabs, { relativePath, content, savedContent: content }]
      activeTabPath = relativePath
    } catch (err: unknown) {
      appendLog(`Open ${relativePath} failed: ${String(err)}`)
    }
  }

  function onTreeSelect(relativePath: string) {
    if (isImageFile(relativePath)) {
      void openImage(relativePath)
    } else {
      void openFile(relativePath)
    }
  }

  function closeImagePreview() {
    imagePreviewPath = null
  }

  function updateActiveContent(content: string) {
    const path = activeTabPath
    if (!path) return
    tabs = tabs.map((t) => (t.relativePath === path ? { ...t, content } : t))
  }

  async function onImportAsset() {
    if (!projectInfo) return
    try {
      const picked = await open({
        multiple: false,
        title: 'Import image to assets',
        filters: IMAGE_DIALOG_FILTERS,
      })
      if (!picked || typeof picked !== 'string') return
      const relativePath = await importBundleAsset(
        projectInfo.project_root,
        selectedSite,
        picked,
      )
      await refreshFileTree()
      appendLog(`Imported asset → ${relativePath}`)
    } catch (err: unknown) {
      appendLog(`Import asset failed: ${String(err)}`)
    }
  }

  async function onRemoveAsset(relativePath: string) {
    if (!projectInfo) return
    try {
      const removed = await removeAssetWithConfirm(
        projectInfo.project_root,
        selectedSite,
        relativePath,
      )
      if (!removed) return
      if (imagePreviewPath === relativePath) {
        imagePreviewPath = null
      }
      await refreshFileTree()
      appendLog(`Removed asset ${relativePath}`)
    } catch (err: unknown) {
      appendLog(`Remove asset failed: ${String(err)}`)
    }
  }

  async function onCopyAssetPath(relativePath: string) {
    try {
      await copyAssetPathToClipboard(relativePath)
      appendLog(`Copied path: ${relativePath}`)
    } catch (err: unknown) {
      appendLog(`Copy path failed: ${String(err)}`)
    }
  }

  function onRenameAsset(relativePath: string) {
    renameAssetPath = relativePath
  }

  function closeRenameModal() {
    renameAssetPath = null
  }

  async function confirmRenameAsset(newName: string) {
    const relativePath = renameAssetPath
    if (!projectInfo || !relativePath) return
    renameAssetPath = null

    try {
      const result = await renameBundleAsset(
        projectInfo.project_root,
        selectedSite,
        relativePath,
        newName,
      )
      const newPath = result.new_relative_path

      tabs = tabs.map((tab) => {
        if (!tab.content.includes(relativePath)) return tab
        const content = tab.content.split(relativePath).join(newPath)
        const savedContent = tab.savedContent.split(relativePath).join(newPath)
        return { ...tab, content, savedContent }
      })

      if (imagePreviewPath === relativePath) {
        imagePreviewPath = newPath
      }

      await refreshFileTree()
      const siteCount = result.updated_sites.length
      appendLog(
        siteCount > 0
          ? `Renamed asset ${relativePath} → ${newPath} (updated ${siteCount} site(s))`
          : `Renamed asset ${relativePath} → ${newPath}`,
      )
    } catch (err: unknown) {
      appendLog(`Rename asset failed: ${String(err)}`)
    }
  }

  async function persistTab(tab: EditorTab): Promise<EditorTab> {
    if (!projectInfo) throw new Error('Open a project first.')
    await writeBundleFile(
      projectInfo.project_root,
      selectedSite,
      tab.relativePath,
      tab.content,
    )
    return { ...tab, savedContent: tab.content }
  }

  function applySavedTabs(saved: EditorTab[]) {
    const savedPaths = new Set(saved.map((t) => t.relativePath))
    tabs = tabs.map((t) =>
      savedPaths.has(t.relativePath)
        ? (saved.find((s) => s.relativePath === t.relativePath) ?? t)
        : t,
    )
  }

  async function onSave() {
    if (!projectInfo) {
      appendLog('Open a project first.')
      return
    }
    const tab = activeTab()
    if (!tab) {
      appendLog('No file open to save.')
      return
    }
    if (!isDirty(tab)) {
      appendLog(`Already saved: ${tab.relativePath}`)
      return
    }
    try {
      const saved = await persistTab(tab)
      applySavedTabs([saved])
      appendLog(`Saved ${saved.relativePath}`)
    } catch (err: unknown) {
      appendLog(`Save failed: ${String(err)}`)
    }
  }

  async function onSaveAll() {
    await saveAllDirtyTabs(true)
  }

  async function saveAllDirtyTabs(logWhenClean = false): Promise<boolean> {
    if (!projectInfo) {
      if (logWhenClean) appendLog('Open a project first.')
      return false
    }
    const dirty = tabs.filter(isDirty)
    if (dirty.length === 0) {
      if (logWhenClean) appendLog('No unsaved changes.')
      return true
    }
    try {
      const saved: EditorTab[] = []
      for (const tab of dirty) {
        saved.push(await persistTab(tab))
      }
      applySavedTabs(saved)
      if (logWhenClean) {
        appendLog(`Saved ${dirty.length} file(s).`)
      }
      return true
    } catch (err: unknown) {
      appendLog(`Save failed: ${String(err)}`)
      return false
    }
  }

  async function saveDirtyTabs(): Promise<boolean> {
    if (!projectInfo) return false
    const dirtyCount = tabs.filter(isDirty).length
    if (dirtyCount === 0) return true
    const ok = await saveAllDirtyTabs(false)
    if (ok) appendLog(`Saved ${dirtyCount} file(s) before build.`)
    return ok
  }

  function onSaveShortcut(event: KeyboardEvent) {
    if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== 's') return
    if (event.shiftKey || event.altKey) return
    event.preventDefault()
    void onSave()
  }

  async function onValidate() {
    if (!projectInfo) {
      appendLog('Open a project first.')
      return
    }
    busy = true
    try {
      const result = await validateSite(projectInfo.project_root, selectedSite, strict)
      setProblemsFromResult(result)
      appendLog(
        result.ok
          ? `Validate OK (${result.warnings.length} warning(s)).`
          : `Validate failed (${result.errors.length} error(s)).`,
      )
    } catch (err: unknown) {
      appendLog(`validate_site failed: ${String(err)}`)
    } finally {
      busy = false
    }
  }

  async function onBuild() {
    if (!projectInfo) {
      appendLog('Open a project first.')
      return
    }
    busy = true
    const buildGeneration = previewGeneration
    try {
      if (!(await saveDirtyTabs())) return

      const result = await buildSite(projectInfo.project_root, selectedSite, strict)
      setProblemsFromResult(result)

      if (!result.ok || !result.output_dir) {
        appendLog('Build failed.')
        return
      }

      if (buildGeneration !== previewGeneration) {
        appendLog('Build finished for a previous site selection; preview not updated.')
        return
      }

      lastOutputDir = result.output_dir
      appendLog(`Build OK → ${result.output_dir}`)

      const preview = await startPreviewServer(result.output_dir, PREVIEW_PORT)
      if (buildGeneration !== previewGeneration) {
        appendLog('Preview ready for a previous site selection; panel not updated.')
        return
      }

      previewUrl = preview.url
      previewRefreshKey += 1
      appendLog(`Preview: ${preview.url}`)
    } catch (err: unknown) {
      appendLog(`build failed: ${String(err)}`)
    } finally {
      busy = false
    }
  }

  async function onOpenOutput() {
    if (!lastOutputDir) {
      await message('Run Build first to generate output.', {
        title: 'No output folder',
        kind: 'info',
      })
      appendLog('No output folder yet — run Build first.')
      return
    }
    try {
      await revealItemInDir(lastOutputDir)
    } catch (err: unknown) {
      const text = String(err)
      await message(text, { title: 'Open output failed', kind: 'error' })
      appendLog(`Open output failed: ${text}`)
    }
  }

  async function onNewSite() {
    if (!projectInfo) {
      appendLog('Open a project first.')
      return
    }
    const raw = window.prompt(
      'New site id (lowercase letters, digits, hyphens):',
      'my-site',
    )
    if (raw === null) return
    const siteId = raw.trim()
    if (!siteId) return

    busy = true
    try {
      const sitePath = await createSiteFromTemplate(projectInfo.project_root, siteId)
      bundles = await listContentBundles(projectInfo.project_root)
      selectedSite = sitePath
      await onSiteChange()
      appendLog(`Created site ${sitePath}`)
    } catch (err: unknown) {
      const text = String(err)
      await message(text, { title: 'New site failed', kind: 'error' })
      appendLog(`New site failed: ${text}`)
    } finally {
      busy = false
    }
  }

  function closeTab(relativePath: string, event: MouseEvent) {
    event.stopPropagation()
    const next = tabs.filter((t) => t.relativePath !== relativePath)
    tabs = next
    if (activeTabPath === relativePath) {
      activeTabPath = next.length > 0 ? next[next.length - 1].relativePath : null
    }
  }

  async function initStudio() {
    try {
      const settings = await getStudioSettings()
      savedWorkspaceLayout = settings.workspace_layout ?? null
      if (savedWorkspaceLayout?.log_collapsed !== undefined && savedWorkspaceLayout.log_collapsed !== null) {
        logCollapsed = savedWorkspaceLayout.log_collapsed
      }
      if (savedWorkspaceLayout?.log_height_px) {
        logHeightPx = savedWorkspaceLayout.log_height_px
      }

      const migratedTheme = settings.theme ? migrateBuiltinThemeId(settings.theme) : null
      const initialTheme =
        migratedTheme ??
        (settings.theme && isBuiltinThemeId(settings.theme)
          ? settings.theme
          : DEFAULT_BUILTIN_THEME)
      themeId = initialTheme
      applyBuiltinTheme(initialTheme, document.documentElement)

      const savedSite = settings.last_selected_site

      if (settings.last_project_root) {
        try {
          await loadProject(settings.last_project_root, savedSite)
          return
        } catch {
          appendLog('Saved project path invalid; using auto-detect.')
        }
      }
      projectInfo = await resolveProjectRoot()
      bundles = await listContentBundles(projectInfo.project_root)
      applySiteSelection(savedSite)
      await refreshFileTree()
      appendLog(`Project: ${projectInfo.project_root}`)
    } catch (err: unknown) {
      appendLog(`Init failed: ${String(err)}`)
    }
  }

  onMount(() => {
    initStudio()

    let unlistenClose: (() => void) | undefined
    let unlistenSave: (() => void) | undefined
    let unlistenSaveAll: (() => void) | undefined

    void listen('studio-save', () => {
      void onSave()
    }).then((fn) => {
      unlistenSave = fn
    })

    void listen('studio-save-all', () => {
      void onSaveAll()
    }).then((fn) => {
      unlistenSaveAll = fn
    })

    getCurrentWindow()
      .onCloseRequested(async () => {
        await persistStudioSettings()
        unlistenClose?.()
        await getCurrentWindow().close()
      })
      .then((fn) => {
        unlistenClose = fn
      })

    return () => {
      unlistenClose?.()
      unlistenSave?.()
      unlistenSaveAll?.()
    }
  })
</script>

<svelte:window onkeydown={onSaveShortcut} />

<div class="studio">
  <header class="toolbar">
    <button type="button" disabled={busy} onclick={onOpenProject}>
      Open project
    </button>

    <button
      type="button"
      disabled={busy || !projectInfo}
      title="Copy content/_template/ to a new bundle"
      onclick={onNewSite}
    >
      New site
    </button>

    <label class="site-select">
      Site
      <select
        bind:value={selectedSite}
        disabled={!projectInfo || bundles.length === 0}
        onchange={onSiteChange}
      >
        {#if bundles.length === 0}
          <option value={DEFAULT_SITE}>{DEFAULT_SITE}</option>
        {:else}
          {#each bundles as bundle (bundle)}
            <option value={bundle}>{bundle}</option>
          {/each}
        {/if}
      </select>
    </label>

    <button type="button" disabled={busy || !projectInfo} onclick={onBuild}>Build</button>
    <button type="button" disabled={busy || !projectInfo} onclick={onValidate}>
      Validate
    </button>

    <label class="strict">
      <input type="checkbox" bind:checked={strict} />
      Strict
    </label>

    <button type="button" disabled={!lastOutputDir} onclick={onOpenOutput}>
      Open output folder
    </button>

    <div class="theme-toggle" role="group" aria-label="App theme">
      {#each BUILTIN_THEME_IDS as id (id)}
        <button
          type="button"
          class:active={themeId === id}
          title={getBuiltinThemeLabel(id)}
          onclick={() => setTheme(id)}
        >
          {getBuiltinThemeLabel(id)}
        </button>
      {/each}
    </div>

    {#if projectInfo}
      <span class="project-path" title={projectInfo.project_root}>
        {projectInfo.project_root}
      </span>
    {/if}
  </header>

  {#if projectInfo}
    <StudioBundleProvider
      projectRoot={projectInfo.project_root}
      sitePath={selectedSite}
      {imageAssets}
      {refreshFileTree}
    >
      <ResizableWorkspace
        bind:sidebarWidth
        bind:previewWidth
        savedLayout={savedWorkspaceLayout}
        onlayoutcommit={onWorkspaceLayoutCommit}
      >
        {#snippet sidebar()}
          <div class="sidebar">
            <FileTree
              entries={fileEntries}
              selectedPath={selectedTreePath()}
              onselect={onTreeSelect}
              onimportasset={onImportAsset}
              onrenameasset={(path) => void onRenameAsset(path)}
              oncopyassetpath={(path) => void onCopyAssetPath(path)}
              onremoveasset={(path) => void onRemoveAsset(path)}
            />
          </div>
        {/snippet}

        {#snippet main()}
          <section class="center">
            <div class="editor-area">
              {#if tabs.length > 0 && !imagePreviewPath}
                <div class="tab-bar" role="tablist">
                  {#each tabs as tab (tab.relativePath)}
                    <button
                      type="button"
                      role="tab"
                      class:active={activeTabPath === tab.relativePath}
                      aria-selected={activeTabPath === tab.relativePath}
                      onclick={() => {
                        imagePreviewPath = null
                        activeTabPath = tab.relativePath
                      }}
                    >
                      {tab.relativePath}
                      {#if isDirty(tab)}<span class="dirty">•</span>{/if}
                      <span
                        class="close"
                        role="button"
                        tabindex="0"
                        aria-label="Close tab"
                        onclick={(e) => closeTab(tab.relativePath, e)}
                        onkeydown={(e) => {
                          if (e.key === 'Enter')
                            closeTab(tab.relativePath, e as unknown as MouseEvent)
                        }}>×</span
                      >
                    </button>
                  {/each}
                </div>
              {/if}

              <div class="editor-pane">
                {#if imagePreviewPath}
                  <ImagePreviewPanel
                    projectRoot={projectInfo!.project_root}
                    sitePath={selectedSite}
                    relativePath={imagePreviewPath}
                    onclose={closeImagePreview}
                    onremove={() => void onRemoveAsset(imagePreviewPath!)}
                  />
                {:else if activeTab()}
                  {@const tab = activeTab()!}
                  {#if supportsFormView(tab.relativePath)}
                    <div class="site-view-bar" role="tablist" aria-label="Editor mode">
                      <button
                        type="button"
                        role="tab"
                        class:active={siteViewForTab(tab) === 'json'}
                        aria-selected={siteViewForTab(tab) === 'json'}
                        onclick={() => setSiteView(tab.relativePath, 'json')}
                      >
                        JSON
                      </button>
                      <button
                        type="button"
                        role="tab"
                        class:active={siteViewForTab(tab) === 'form'}
                        aria-selected={siteViewForTab(tab) === 'form'}
                        onclick={() => setSiteView(tab.relativePath, 'form')}
                      >
                        Form
                      </button>
                    </div>
                  {/if}
                  <div class="editor-body">
                  {#key `${tab.relativePath}:${siteViewForTab(tab)}`}
                    {#if siteViewForTab(tab) === 'form'}
                      {#if isSiteJson(tab.relativePath)}
                        <SiteFormEditor value={tab.content} onchange={updateActiveContent} />
                      {:else if isPageJson(tab.relativePath)}
                        <PageFormEditor value={tab.content} onchange={updateActiveContent} />
                      {:else}
                        <JsonEditor
                          relativePath={tab.relativePath}
                          value={tab.content}
                          onchange={updateActiveContent}
                        />
                      {/if}
                    {:else}
                      <JsonEditor
                        relativePath={tab.relativePath}
                        value={tab.content}
                        onchange={updateActiveContent}
                      />
                    {/if}
                  {/key}
                  </div>
                {:else}
                  <p class="editor-placeholder">Select a JSON or image file from the tree.</p>
                {/if}
              </div>
            </div>
            <ProblemsPanel items={problems} />
          </section>
        {/snippet}

        {#snippet preview()}
          <div class="preview">
            <PreviewPanel
              previewUrl={previewUrl}
              refreshKey={previewRefreshKey}
              onrefresh={() => (previewRefreshKey += 1)}
            />
          </div>
        {/snippet}
      </ResizableWorkspace>
    </StudioBundleProvider>
  {:else}
    <div class="loading-placeholder">Loading project…</div>
  {/if}

  <LogPanel
    lines={logs}
    collapsed={logCollapsed}
    heightPx={logHeightPx}
    onTogglePanel={toggleLogPanel}
    onheightchange={onLogHeightChange}
  />

  {#if renameAssetPath}
    <AssetRenameModal
      open={true}
      relativePath={renameAssetPath}
      onrename={(name) => void confirmRenameAsset(name)}
      onclose={closeRenameModal}
    />
  {/if}
</div>

<style>
  .studio {
    display: flex;
    flex-direction: column;
    height: 100vh;
    min-height: 0;
    color: var(--color-text-primary);
    background: var(--color-bg-root);
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 0.65rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-surface-1);
    flex-shrink: 0;
  }

  .toolbar button {
    padding: 0.35rem 0.65rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    background: var(--color-hover);
    font-size: 0.85rem;
    cursor: pointer;
  }

  .toolbar button:hover:not(:disabled) {
    background: var(--color-pressed);
  }

  .toolbar button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .theme-toggle {
    display: flex;
    gap: 0.2rem;
    margin-left: auto;
  }

  .theme-toggle button {
    font-size: 0.78rem;
    padding: 0.3rem 0.5rem;
  }

  .theme-toggle button.active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 18%, var(--color-surface-1));
    font-weight: 600;
  }

  .site-select {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85rem;
  }

  select {
    min-width: 10rem;
  }

  .strict {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.85rem;
  }

  .project-path {
    max-width: 40%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    font-family: ui-monospace, Consolas, monospace;
  }

  .loading-placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-secondary);
    font-size: 0.9rem;
  }

  .sidebar {
    height: 100%;
    border-right: 1px solid var(--color-border-subtle);
    background: var(--color-surface-1);
    min-height: 0;
    overflow: hidden;
  }

  .center {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
    background: var(--color-surface-1);
  }

  .editor-area {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .tab-bar {
    display: flex;
    flex-wrap: nowrap;
    overflow-x: auto;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-statusbar-bg);
    flex-shrink: 0;
  }

  .tab-bar button {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.35rem 0.65rem;
    border: none;
    border-right: 1px solid var(--color-border-subtle);
    background: transparent;
    font-size: 0.78rem;
    font-family: ui-monospace, Consolas, monospace;
    cursor: pointer;
    white-space: nowrap;
  }

  .tab-bar button:hover {
    background: var(--color-hover);
  }

  .tab-bar button.active {
    background: var(--color-surface-1);
    font-weight: 600;
  }

  .dirty {
    color: var(--color-accent);
  }

  .close {
    margin-left: 0.15rem;
    opacity: 0.5;
    cursor: pointer;
  }

  .close:hover {
    opacity: 1;
  }

  .editor-pane {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .editor-body {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .site-view-bar {
    display: flex;
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-statusbar-bg);
  }

  .site-view-bar button {
    padding: 0.35rem 0.85rem;
    border: none;
    border-right: 1px solid var(--color-border-subtle);
    background: transparent;
    font-size: 0.8rem;
    cursor: pointer;
  }

  .site-view-bar button:hover {
    background: var(--color-hover);
  }

  .site-view-bar button.active {
    background: var(--color-surface-1);
    font-weight: 600;
  }

  .editor-pane :global(.editor-host),
  .editor-pane :global(.site-form),
  .editor-pane :global(.page-form),
  .editor-pane :global(.image-preview) {
    flex: 1;
    min-height: 0;
  }

  .editor-placeholder {
    margin: 2rem 1rem;
    color: var(--color-text-secondary);
    font-size: 0.9rem;
  }

  .preview {
    height: 100%;
    border-left: 1px solid var(--color-border-subtle);
    min-height: 0;
    overflow: hidden;
  }
</style>

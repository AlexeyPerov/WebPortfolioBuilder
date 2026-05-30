<script lang="ts">
  import { onMount } from 'svelte'
  import { listen } from '@tauri-apps/api/event'
  import { message, open } from '@tauri-apps/plugin-dialog'
  import { revealItemInDir } from '@tauri-apps/plugin-opener'
  import BuildLog from './components/BuildLog.svelte'
  import FileTree from './components/FileTree.svelte'
  import JsonEditor from './components/JsonEditor.svelte'
  import SiteFormEditor from './components/SiteFormEditor.svelte'
  import PreviewPanel from './components/PreviewPanel.svelte'
  import ProblemsPanel from './components/ProblemsPanel.svelte'
  import {
    AUTO_REBUILD_DEBOUNCE_MS,
    buildSite,
    createSiteFromTemplate,
    getStudioSettings,
    listBundleFiles,
    listContentBundles,
    projectInfoForRoot,
    readBundleFile,
    resolveProjectRoot,
    saveStudioSettings,
    setAutoRebuild,
    startPreviewServer,
    validateSite,
    writeBundleFile,
    type BuildSiteResult,
    type BundleFileEntry,
    type BuiltinThemeId,
    type Diagnostic,
    type ProjectRootInfo,
    type ValidateSiteResult,
    type WatchRebuildComplete,
  } from './lib/studio-api'
  import {
    applyBuiltinTheme,
    BUILTIN_THEME_IDS,
    DEFAULT_BUILTIN_THEME,
    getBuiltinThemeLabel,
    isBuiltinThemeId,
  } from './lib/styles/themeTokens'

  const DEFAULT_SITE = 'content/kometa'
  const PREVIEW_PORT = 8080

  type SiteEditorView = 'json' | 'form'

  type EditorTab = {
    relativePath: string
    content: string
    savedContent: string
    siteView?: SiteEditorView
  }

  function isSiteJson(relativePath: string) {
    return relativePath.replace(/\\/g, '/') === 'site.json'
  }

  function siteViewForTab(tab: EditorTab): SiteEditorView {
    return tab.siteView ?? 'json'
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
  let problems = $state<Diagnostic[]>([])
  let logs = $state<string[]>([])
  let lastOutputDir = $state<string | null>(null)
  let previewUrl = $state<string | null>(null)
  let previewRefreshKey = $state(0)
  let busy = $state(false)
  let autoRebuild = $state(false)
  let themeId = $state<BuiltinThemeId>(DEFAULT_BUILTIN_THEME)

  let autoSaveTimer: ReturnType<typeof setTimeout> | null = null

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
    theme?: BuiltinThemeId
  } = {}) {
    await saveStudioSettings({
      last_project_root:
        overrides.last_project_root !== undefined
          ? overrides.last_project_root
          : projectInfo?.project_root,
      theme: overrides.theme ?? themeId,
    })
  }

  async function persistProjectRoot(root: string) {
    await persistStudioSettings({ last_project_root: root })
  }

  function setTheme(id: BuiltinThemeId) {
    themeId = id
    applyBuiltinTheme(id, document.documentElement)
    void persistStudioSettings({ theme: id })
  }

  async function loadProject(root: string) {
    projectInfo = await projectInfoForRoot(root)
    await persistProjectRoot(projectInfo.project_root)
    bundles = await listContentBundles(projectInfo.project_root)
    if (bundles.includes(DEFAULT_SITE)) {
      selectedSite = DEFAULT_SITE
    } else if (bundles.length > 0) {
      selectedSite = bundles[0]
    }
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
      await loadProject(picked)
    } catch (err: unknown) {
      appendLog(`Open project failed: ${String(err)}`)
    } finally {
      busy = false
    }
  }

  async function onSiteChange() {
    tabs = []
    activeTabPath = null
    problems = []
    await refreshFileTree()
  }

  async function openFile(relativePath: string) {
    if (!projectInfo) return
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

  function updateActiveContent(content: string) {
    const path = activeTabPath
    if (!path) return
    tabs = tabs.map((t) => (t.relativePath === path ? { ...t, content } : t))
    scheduleAutoSave()
  }

  function scheduleAutoSave() {
    if (!autoRebuild || !projectInfo) return
    if (autoSaveTimer) clearTimeout(autoSaveTimer)
    autoSaveTimer = setTimeout(async () => {
      autoSaveTimer = null
      if (tabs.some(isDirty)) {
        await saveDirtyTabs()
      }
    }, AUTO_REBUILD_DEBOUNCE_MS)
  }

  async function syncAutoRebuildWatcher() {
    if (!projectInfo) return
    try {
      await setAutoRebuild(
        autoRebuild,
        projectInfo.project_root,
        selectedSite,
        strict,
        PREVIEW_PORT,
      )
    } catch (err: unknown) {
      appendLog(`Auto-rebuild watcher failed: ${String(err)}`)
    }
  }

  $effect(() => {
    if (!projectInfo) return
    autoRebuild
    selectedSite
    strict
    void syncAutoRebuildWatcher()
  })

  async function saveDirtyTabs(): Promise<boolean> {
    if (!projectInfo) return false
    const dirty = tabs.filter(isDirty)
    if (dirty.length === 0) return true
    try {
      const saved: EditorTab[] = []
      for (const tab of dirty) {
        await writeBundleFile(
          projectInfo.project_root,
          selectedSite,
          tab.relativePath,
          tab.content,
        )
        saved.push({ ...tab, savedContent: tab.content })
      }
      const savedPaths = new Set(saved.map((t) => t.relativePath))
      tabs = tabs.map((t) =>
        savedPaths.has(t.relativePath)
          ? (saved.find((s) => s.relativePath === t.relativePath) ?? t)
          : t,
      )
      appendLog(`Saved ${dirty.length} file(s) before build.`)
      return true
    } catch (err: unknown) {
      appendLog(`Save failed: ${String(err)}`)
      return false
    }
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
    const wasAutoRebuild = autoRebuild
    if (wasAutoRebuild) {
      await setAutoRebuild(false, projectInfo.project_root, selectedSite, strict, PREVIEW_PORT)
    }
    try {
      if (!(await saveDirtyTabs())) return

      const result = await buildSite(projectInfo.project_root, selectedSite, strict)
      setProblemsFromResult(result)

      if (!result.ok || !result.output_dir) {
        appendLog('Build failed.')
        return
      }

      lastOutputDir = result.output_dir
      appendLog(`Build OK → ${result.output_dir}`)

      const preview = await startPreviewServer(result.output_dir, PREVIEW_PORT)
      previewUrl = preview.url
      previewRefreshKey += 1
      appendLog(`Preview: ${preview.url}`)
    } catch (err: unknown) {
      appendLog(`build failed: ${String(err)}`)
    } finally {
      if (wasAutoRebuild && projectInfo) {
        await setAutoRebuild(
          true,
          projectInfo.project_root,
          selectedSite,
          strict,
          PREVIEW_PORT,
        )
      }
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
      const initialTheme =
        settings.theme && isBuiltinThemeId(settings.theme)
          ? settings.theme
          : DEFAULT_BUILTIN_THEME
      themeId = initialTheme
      applyBuiltinTheme(initialTheme, document.documentElement)

      if (settings.last_project_root) {
        try {
          await loadProject(settings.last_project_root)
          return
        } catch {
          appendLog('Saved project path invalid; using auto-detect.')
        }
      }
      projectInfo = await resolveProjectRoot()
      bundles = await listContentBundles(projectInfo.project_root)
      if (bundles.includes(DEFAULT_SITE)) {
        selectedSite = DEFAULT_SITE
      } else if (bundles.length > 0) {
        selectedSite = bundles[0]
      }
      await refreshFileTree()
      appendLog(`Project: ${projectInfo.project_root}`)
    } catch (err: unknown) {
      appendLog(`Init failed: ${String(err)}`)
    }
  }

  onMount(() => {
    initStudio()

    let unlisten: (() => void) | undefined
    listen<WatchRebuildComplete>('watch-rebuild-complete', (event) => {
      const { build, preview } = event.payload
      setProblemsFromResult(build)
      if (build.ok && build.output_dir) {
        lastOutputDir = build.output_dir
        appendLog(`Auto-rebuild OK → ${build.output_dir}`)
        if (preview) {
          previewUrl = preview.url
          previewRefreshKey += 1
          appendLog(`Preview: ${preview.url}`)
        }
      } else {
        appendLog(`Auto-rebuild failed (${build.errors.length} error(s)).`)
      }
    }).then((fn) => {
      unlisten = fn
    })

    return () => {
      unlisten?.()
      if (autoSaveTimer) clearTimeout(autoSaveTimer)
    }
  })
</script>

<div class="studio">
  <header class="toolbar">
    <button type="button" class="primary" disabled={busy} onclick={onOpenProject}>
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

    <label class="strict" title="Watch content bundle and rebuild after saves (500 ms debounce)">
      <input type="checkbox" bind:checked={autoRebuild} disabled={!projectInfo} />
      Auto-rebuild
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

  <div class="workspace">
    <aside class="sidebar">
      <FileTree
        entries={fileEntries}
        selectedPath={activeTabPath}
        onselect={(path) => openFile(path)}
      />
    </aside>

    <section class="center">
      <div class="editor-area">
        {#if tabs.length > 0}
          <div class="tab-bar" role="tablist">
            {#each tabs as tab (tab.relativePath)}
              <button
                type="button"
                role="tab"
                class:active={activeTabPath === tab.relativePath}
                aria-selected={activeTabPath === tab.relativePath}
                onclick={() => (activeTabPath = tab.relativePath)}
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
                    if (e.key === 'Enter') closeTab(tab.relativePath, e as unknown as MouseEvent)
                  }}>×</span
                >
              </button>
            {/each}
          </div>
        {/if}

        <div class="editor-pane">
          {#if activeTab()}
            {@const tab = activeTab()!}
            {#if isSiteJson(tab.relativePath)}
              <div class="site-view-bar" role="tablist" aria-label="site.json editor mode">
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
            {#key `${tab.relativePath}:${siteViewForTab(tab)}`}
              {#if isSiteJson(tab.relativePath) && siteViewForTab(tab) === 'form'}
                <SiteFormEditor value={tab.content} onchange={updateActiveContent} />
              {:else}
                <JsonEditor
                  relativePath={tab.relativePath}
                  value={tab.content}
                  onchange={updateActiveContent}
                />
              {/if}
            {/key}
          {:else}
            <p class="editor-placeholder">Select a JSON file from the tree.</p>
          {/if}
        </div>
      </div>
      <ProblemsPanel items={problems} />
    </section>

    <aside class="preview">
      <PreviewPanel
        previewUrl={previewUrl}
        refreshKey={previewRefreshKey}
        onrefresh={() => (previewRefreshKey += 1)}
      />
    </aside>
  </div>

  <footer class="log-footer">
    <BuildLog lines={logs} />
  </footer>
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

  .toolbar button.primary {
    border-color: var(--color-accent);
    background: var(--color-accent);
    color: #fff;
  }

  .theme-toggle {
    display: flex;
    gap: 0.2rem;
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
    margin-left: auto;
    max-width: 40%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    font-family: ui-monospace, Consolas, monospace;
  }

  .workspace {
    display: grid;
    grid-template-columns: 12rem 1fr minmax(280px, 38%);
    flex: 1;
    min-height: 0;
  }

  .sidebar {
    border-right: 1px solid var(--color-border-subtle);
    background: var(--color-surface-1);
    min-height: 0;
    overflow: hidden;
  }

  .center {
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
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
  .editor-pane :global(.site-form) {
    flex: 1;
    min-height: 0;
  }

  .editor-placeholder {
    margin: 2rem 1rem;
    color: var(--color-text-secondary);
    font-size: 0.9rem;
  }

  .preview {
    border-left: 1px solid var(--color-border-subtle);
    min-height: 0;
    overflow: hidden;
  }

  .log-footer {
    flex-shrink: 0;
    height: 7rem;
    min-height: 7rem;
  }
</style>

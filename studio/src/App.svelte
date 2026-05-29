<script lang="ts">
  import { onMount } from 'svelte'
  import { listen } from '@tauri-apps/api/event'
  import { open } from '@tauri-apps/plugin-dialog'
  import { openPath } from '@tauri-apps/plugin-opener'
  import BuildLog from './components/BuildLog.svelte'
  import FileTree from './components/FileTree.svelte'
  import JsonEditor from './components/JsonEditor.svelte'
  import PreviewPanel from './components/PreviewPanel.svelte'
  import ProblemsPanel from './components/ProblemsPanel.svelte'
  import {
    AUTO_REBUILD_DEBOUNCE_MS,
    buildSite,
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
    type Diagnostic,
    type ProjectRootInfo,
    type ValidateSiteResult,
    type WatchRebuildComplete,
  } from './lib/studio-api'

  const DEFAULT_SITE = 'content/kometa'
  const PREVIEW_PORT = 8080

  type EditorTab = {
    relativePath: string
    content: string
    savedContent: string
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

  async function persistProjectRoot(root: string) {
    await saveStudioSettings({ last_project_root: root })
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
      appendLog('No output folder yet — run Build first.')
      return
    }
    try {
      await openPath(lastOutputDir)
    } catch (err: unknown) {
      appendLog(`Open output failed: ${String(err)}`)
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
            {#key tab.relativePath}
              <JsonEditor
                relativePath={tab.relativePath}
                value={tab.content}
                onchange={updateActiveContent}
              />
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
    color: #1a1a1a;
    background: #f6f7f9;
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 0.65rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid #d8dee6;
    background: #fff;
    flex-shrink: 0;
  }

  .toolbar button {
    padding: 0.35rem 0.65rem;
    border: 1px solid #c5cdd8;
    border-radius: 0.35rem;
    background: #f0f2f5;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .toolbar button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .toolbar button.primary {
    border-color: #3d5a80;
    background: #3d5a80;
    color: #fff;
  }

  .site-select {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85rem;
  }

  select {
    min-width: 10rem;
    padding: 0.3rem 0.45rem;
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
    color: #5c6570;
    font-family: ui-monospace, Consolas, monospace;
  }

  .workspace {
    display: grid;
    grid-template-columns: 12rem 1fr minmax(280px, 38%);
    flex: 1;
    min-height: 0;
  }

  .sidebar {
    border-right: 1px solid #d8dee6;
    background: #fff;
    min-height: 0;
    overflow: hidden;
  }

  .center {
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
    background: #fff;
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
    border-bottom: 1px solid #e8ecf2;
    background: #fafbfc;
    flex-shrink: 0;
  }

  .tab-bar button {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.35rem 0.65rem;
    border: none;
    border-right: 1px solid #e8ecf2;
    background: transparent;
    font-size: 0.78rem;
    font-family: ui-monospace, Consolas, monospace;
    cursor: pointer;
    white-space: nowrap;
  }

  .tab-bar button.active {
    background: #fff;
    font-weight: 600;
  }

  .dirty {
    color: #c45c26;
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
    flex: 1;
    min-height: 0;
  }

  .editor-placeholder {
    margin: 2rem 1rem;
    color: #8a939e;
    font-size: 0.9rem;
  }

  .preview {
    border-left: 1px solid #d8dee6;
    min-height: 0;
    overflow: hidden;
  }

  .log-footer {
    flex-shrink: 0;
    height: 7rem;
    min-height: 7rem;
  }
</style>

<script lang="ts">
  import {
    buildSite,
    listContentBundles,
    resolveProjectRoot,
    startPreviewServer,
    stopPreviewServer,
    validateSite,
    type BuildSiteResult,
    type ProjectRootInfo,
    type ValidateSiteResult,
  } from './lib/studio-api'

  const DEFAULT_SITE = 'content/kometa'
  const PREVIEW_PORT = 8080

  let projectInfo = $state<ProjectRootInfo | null>(null)
  let bundles = $state<string[]>([])
  let selectedSite = $state(DEFAULT_SITE)
  let strict = $state(false)
  let logs = $state<string[]>([])
  let lastOutputDir = $state<string | null>(null)
  let previewUrl = $state<string | null>(null)

  function appendLog(line: string) {
    const stamp = new Date().toLocaleTimeString()
    logs = [`[${stamp}] ${line}`, ...logs].slice(0, 200)
  }

  function formatResult(label: string, payload: unknown) {
    appendLog(`${label}: ${JSON.stringify(payload, null, 2)}`)
  }

  async function ensureProject() {
    if (!projectInfo) {
      projectInfo = await resolveProjectRoot()
      appendLog(`Project root: ${projectInfo.project_root}`)
    }
    return projectInfo
  }

  async function onResolveRoot() {
    try {
      projectInfo = await resolveProjectRoot()
      formatResult('resolve_project_root', projectInfo)
    } catch (err: unknown) {
      appendLog(`resolve_project_root failed: ${String(err)}`)
    }
  }

  async function onListBundles() {
    try {
      const info = await ensureProject()
      bundles = await listContentBundles(info.project_root)
      formatResult('list_content_bundles', bundles)
      if (bundles.includes(DEFAULT_SITE)) {
        selectedSite = DEFAULT_SITE
      } else if (bundles.length > 0) {
        selectedSite = bundles[0]
      }
    } catch (err: unknown) {
      appendLog(`list_content_bundles failed: ${String(err)}`)
    }
  }

  async function onValidate() {
    try {
      const info = await ensureProject()
      const result: ValidateSiteResult = await validateSite(
        info.project_root,
        selectedSite,
        strict,
      )
      formatResult('validate_site', result)
    } catch (err: unknown) {
      appendLog(`validate_site failed: ${String(err)}`)
    }
  }

  async function onBuild() {
    try {
      const info = await ensureProject()
      const result: BuildSiteResult = await buildSite(
        info.project_root,
        selectedSite,
        strict,
      )
      formatResult('build_site', result)
      if (result.ok && result.output_dir) {
        lastOutputDir = result.output_dir
      }
    } catch (err: unknown) {
      appendLog(`build_site failed: ${String(err)}`)
    }
  }

  async function onStartPreview() {
    try {
      if (!lastOutputDir) {
        appendLog('start_preview_server: build first to set output_dir')
        return
      }
      const info = await startPreviewServer(lastOutputDir, PREVIEW_PORT)
      previewUrl = info.url
      formatResult('start_preview_server', info)
    } catch (err: unknown) {
      appendLog(`start_preview_server failed: ${String(err)}`)
    }
  }

  async function onStopPreview() {
    try {
      await stopPreviewServer()
      previewUrl = null
      appendLog('stop_preview_server: ok')
    } catch (err: unknown) {
      appendLog(`stop_preview_server failed: ${String(err)}`)
    }
  }

  async function onBuildAndPreview() {
    await onBuild()
    if (lastOutputDir) {
      await onStartPreview()
    }
  }

  $effect(() => {
    onResolveRoot()
      .then(() => onListBundles())
      .catch((err: unknown) => appendLog(`init failed: ${String(err)}`))
  })
</script>

<main>
  <header>
    <h1>Portfolio Website Builder</h1>
    <p class="subtitle">Phase 2.2 — invoke API dev panel</p>
  </header>

  {#if projectInfo}
    <section class="paths">
      <p>
        <span class="label">Project</span>
        <code>{projectInfo.project_root}</code>
      </p>
      <p>
        <span class="label">Template</span>
        <code>{projectInfo.template_dir}</code>
      </p>
    </section>
  {/if}

  <section class="controls">
    <label class="strict">
      <input type="checkbox" bind:checked={strict} />
      Strict
    </label>

    <label>
      Site bundle
      <select bind:value={selectedSite}>
        {#if bundles.length === 0}
          <option value={DEFAULT_SITE}>{DEFAULT_SITE}</option>
        {:else}
          {#each bundles as bundle (bundle)}
            <option value={bundle}>{bundle}</option>
          {/each}
        {/if}
      </select>
    </label>

    <div class="buttons">
      <button type="button" onclick={onResolveRoot}>Resolve root</button>
      <button type="button" onclick={onListBundles}>List bundles</button>
      <button type="button" onclick={onValidate}>Validate</button>
      <button type="button" onclick={onBuild}>Build</button>
      <button type="button" onclick={onStartPreview}>Start preview</button>
      <button type="button" onclick={onStopPreview}>Stop preview</button>
      <button type="button" class="primary" onclick={onBuildAndPreview}>Build + preview</button>
    </div>
  </section>

  {#if previewUrl}
    <p class="preview-link">
      Preview: <a href={previewUrl} target="_blank" rel="noreferrer">{previewUrl}</a>
    </p>
  {/if}

  {#if lastOutputDir}
    <p class="output-dir">
      Last output: <code>{lastOutputDir}</code>
    </p>
  {/if}

  <section class="log-panel">
    <h2>Log</h2>
    <pre>{logs.length > 0 ? logs.join('\n\n') : 'No log entries yet.'}</pre>
  </section>
</main>

<style>
  main {
    box-sizing: border-box;
    min-height: 100vh;
    padding: 1.5rem 1.25rem 2rem;
    font-family:
      system-ui,
      -apple-system,
      sans-serif;
    color: #1a1a1a;
    background: #f6f7f9;
  }

  header {
    margin-bottom: 1rem;
  }

  h1 {
    margin: 0 0 0.25rem;
    font-size: 1.5rem;
    font-weight: 650;
  }

  .subtitle {
    margin: 0;
    color: #5c6570;
    font-size: 0.95rem;
  }

  h2 {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #5c6570;
  }

  .paths,
  .controls,
  .log-panel {
    margin-bottom: 1rem;
    padding: 0.85rem 1rem;
    border: 1px solid #d8dee6;
    border-radius: 0.5rem;
    background: #fff;
  }

  .paths p {
    margin: 0.35rem 0;
  }

  .label {
    display: block;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #5c6570;
    margin-bottom: 0.15rem;
  }

  code {
    font-size: 0.85rem;
    word-break: break-all;
  }

  .controls label {
    display: block;
    margin-bottom: 0.65rem;
    font-size: 0.9rem;
  }

  .strict {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  select {
    display: block;
    margin-top: 0.25rem;
    min-width: 14rem;
    padding: 0.35rem 0.5rem;
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  button {
    padding: 0.4rem 0.65rem;
    border: 1px solid #c5cdd8;
    border-radius: 0.35rem;
    background: #f0f2f5;
    font-size: 0.85rem;
    cursor: pointer;
  }

  button:hover {
    background: #e4e8ee;
  }

  button.primary {
    border-color: #3d5a80;
    background: #3d5a80;
    color: #fff;
  }

  button.primary:hover {
    background: #2f4763;
  }

  .preview-link,
  .output-dir {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
  }

  .log-panel pre {
    margin: 0;
    max-height: 22rem;
    overflow: auto;
    font-size: 0.75rem;
    line-height: 1.35;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>

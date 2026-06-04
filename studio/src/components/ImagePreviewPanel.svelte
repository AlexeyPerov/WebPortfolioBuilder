<script lang="ts">
  import { readBundleImage, type BundleImagePreview } from '../lib/studio-api'

  type Props = {
    projectRoot: string
    sitePath: string
    relativePath: string
    onclose?: () => void
    onremove?: () => void
  }

  let { projectRoot, sitePath, relativePath, onclose, onremove }: Props = $props()

  let preview = $state<BundleImagePreview | null>(null)
  let error = $state<string | null>(null)
  let loading = $state(true)

  async function loadImage(path: string) {
    loading = true
    error = null
    preview = null
    try {
      preview = await readBundleImage(projectRoot, sitePath, path)
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err)
    } finally {
      loading = false
    }
  }

  $effect(() => {
    void loadImage(relativePath)
  })
</script>

<section class="image-preview" aria-label="Image preview">
  <header class="preview-header">
    <div class="title-wrap">
      <h2>{relativePath}</h2>
    </div>
    <div class="header-actions">
      {#if onremove}
        <button type="button" class="remove-btn" onclick={() => onremove?.()}>Remove</button>
      {/if}
      <button type="button" class="close-btn" onclick={() => onclose?.()}>Close</button>
    </div>
  </header>

  <div class="preview-body">
    {#if loading}
      <p class="status">Loading image…</p>
    {:else if error}
      <p class="status error" role="alert">{error}</p>
    {:else if preview}
      <img src={preview.data_url} alt={relativePath} draggable="false" />
    {/if}
  </div>
</section>

<style>
  .image-preview {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    background: var(--color-surface-1);
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    flex-shrink: 0;
    padding: 0.45rem 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-statusbar-bg);
  }

  .title-wrap h2 {
    margin: 0;
    font-size: 0.82rem;
    font-weight: 600;
    font-family: ui-monospace, Consolas, monospace;
    word-break: break-all;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .remove-btn,
  .close-btn {
    flex-shrink: 0;
    padding: 0.3rem 0.65rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    background: var(--color-hover);
    font-size: 0.8rem;
    cursor: pointer;
  }

  .remove-btn {
    color: #e06c75;
    border-color: color-mix(in srgb, #e06c75 40%, var(--color-border-subtle));
  }

  .remove-btn:hover,
  .close-btn:hover {
    background: var(--color-pressed);
  }

  .preview-body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
  }

  .preview-body img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 0.25rem;
    box-shadow: 0 0 0 1px var(--color-border-subtle);
  }

  .status {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.9rem;
  }

  .status.error {
    color: #e06c75;
  }
</style>

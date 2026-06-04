<script lang="ts">
  import { readBundleImage } from '../lib/studio-api'

  type Props = {
    open: boolean
    projectRoot: string
    sitePath: string
    assets: string[]
    onselect?: (relativePath: string) => void
    onclose?: () => void
  }

  let { open, projectRoot, sitePath, assets, onselect, onclose }: Props = $props()

  let thumbnails = $state<Record<string, string>>({})

  async function loadThumbnail(relativePath: string) {
    if (thumbnails[relativePath]) return
    try {
      const preview = await readBundleImage(projectRoot, sitePath, relativePath)
      thumbnails = { ...thumbnails, [relativePath]: preview.data_url }
    } catch {
      // Thumbnail optional — list still works without it
    }
  }

  $effect(() => {
    if (!open) return
    for (const path of assets) {
      void loadThumbnail(path)
    }
  })

  function select(path: string) {
    onselect?.(path)
    onclose?.()
  }

  function onBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) onclose?.()
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') onclose?.()
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Select asset"
    tabindex="-1"
    onclick={onBackdropClick}
    onkeydown={onKeydown}
  >
    <div class="modal">
      <header class="modal-header">
        <h2>Select image</h2>
        <button type="button" class="close-btn" onclick={() => onclose?.()}>×</button>
      </header>
      {#if assets.length === 0}
        <p class="empty">No images in assets folder yet.</p>
      {:else}
        <ul class="asset-list">
          {#each assets as path (path)}
            <li>
              <button type="button" class="asset-row" onclick={() => select(path)}>
                {#if thumbnails[path]}
                  <img src={thumbnails[path]} alt="" class="thumb" />
                {:else}
                  <span class="thumb placeholder" aria-hidden="true"></span>
                {/if}
                <span class="path">{path}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.45);
    padding: 1rem;
  }

  .modal {
    width: min(28rem, 100%);
    max-height: min(24rem, 80vh);
    display: flex;
    flex-direction: column;
    background: var(--color-surface-1);
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.5rem;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .close-btn {
    padding: 0.15rem 0.45rem;
    border: none;
    background: transparent;
    font-size: 1.1rem;
    line-height: 1;
    cursor: pointer;
    color: var(--color-text-secondary);
  }

  .close-btn:hover {
    color: var(--color-text-primary);
  }

  .empty {
    margin: 0;
    padding: 1.25rem 0.75rem;
    color: var(--color-text-secondary);
    font-size: 0.85rem;
  }

  .asset-list {
    margin: 0;
    padding: 0.35rem 0;
    list-style: none;
    overflow: auto;
  }

  .asset-row {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    width: 100%;
    padding: 0.35rem 0.75rem;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    font: inherit;
    color: inherit;
  }

  .asset-row:hover {
    background: var(--color-hover);
  }

  .thumb {
    width: 2.25rem;
    height: 2.25rem;
    object-fit: cover;
    border-radius: 0.25rem;
    flex-shrink: 0;
    border: 1px solid var(--color-border-subtle);
  }

  .thumb.placeholder {
    display: inline-block;
    background: var(--color-hover);
  }

  .path {
    font-family: ui-monospace, Consolas, monospace;
    font-size: 0.78rem;
    word-break: break-all;
  }
</style>

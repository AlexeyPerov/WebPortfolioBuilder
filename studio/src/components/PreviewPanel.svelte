<script lang="ts">
  type Viewport = 'desktop' | 'phone'

  type Props = {
    previewUrl: string | null
    refreshKey: number
    onrefresh?: () => void
  }

  let { previewUrl, refreshKey, onrefresh }: Props = $props()
  let viewport = $state<Viewport>('desktop')

  const iframeSrc = $derived(
    previewUrl ? `${previewUrl}${previewUrl.includes('?') ? '&' : '?'}_r=${refreshKey}` : null,
  )
</script>

<section class="preview-panel" aria-label="Site preview">
  <header>
    <h2>Preview</h2>
    <div class="actions">
      <button type="button" disabled={!previewUrl} onclick={() => onrefresh?.()}>
        Refresh
      </button>
      <div class="viewport" role="group" aria-label="Viewport width">
        <button
          type="button"
          class:active={viewport === 'desktop'}
          onclick={() => (viewport = 'desktop')}
        >
          Desktop
        </button>
        <button
          type="button"
          class:active={viewport === 'phone'}
          onclick={() => (viewport = 'phone')}
        >
          Phone
        </button>
      </div>
    </div>
  </header>
  <div class="frame-wrap" class:phone={viewport === 'phone'}>
    {#if iframeSrc}
      <iframe title="Site preview" src={iframeSrc}></iframe>
    {:else}
      <p class="placeholder">Build the site to load preview over HTTP.</p>
    {/if}
  </div>
</section>

<style>
  .preview-panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    background: var(--color-bg-root);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.4rem 0.65rem;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-surface-1);
  }

  h2 {
    margin: 0;
    font-size: 0.72rem;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-secondary);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  button {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.3rem;
    background: var(--color-hover);
    font-size: 0.78rem;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: var(--color-pressed);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.active {
    border-color: var(--color-accent);
    background: var(--color-accent);
    color: #fff;
  }

  .viewport {
    display: flex;
    gap: 0.2rem;
  }

  .frame-wrap {
    flex: 1;
    min-height: 0;
    display: flex;
    justify-content: center;
    padding: 0.5rem;
    overflow: auto;
  }

  .frame-wrap iframe {
    width: 100%;
    height: 100%;
    min-height: 12rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    background: #fff;
  }

  .frame-wrap.phone iframe {
    width: 390px;
    max-width: 100%;
  }

  .placeholder {
    margin: auto;
    font-size: 0.85rem;
    color: var(--color-text-secondary);
    text-align: center;
    padding: 1rem;
  }
</style>

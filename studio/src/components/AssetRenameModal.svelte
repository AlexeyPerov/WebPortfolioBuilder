<script lang="ts">
  type Props = {
    open: boolean
    relativePath: string
    onrename?: (newName: string) => void
    onclose?: () => void
  }

  let { open, relativePath, onrename, onclose }: Props = $props()

  let nameInput = $state('')
  let inputEl = $state<HTMLInputElement | null>(null)

  function assetFileName(path: string): string {
    const parts = path.replace(/\\/g, '/').split('/')
    return parts[parts.length - 1] ?? path
  }

  const currentName = $derived(assetFileName(relativePath))
  const canRename = $derived(
    nameInput.trim().length > 0 && nameInput.trim() !== currentName,
  )

  $effect(() => {
    if (!open) return
    nameInput = currentName
    queueMicrotask(() => {
      inputEl?.focus()
      inputEl?.select()
    })
  })

  function submit() {
    if (!canRename) return
    onrename?.(nameInput.trim())
  }

  function onBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) onclose?.()
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') onclose?.()
  }

  function onInputKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault()
      submit()
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Rename asset"
    tabindex="-1"
    onclick={onBackdropClick}
    onkeydown={onKeydown}
  >
    <div class="modal">
      <header class="modal-header">
        <h2>Rename asset</h2>
        <button type="button" class="close-btn" onclick={() => onclose?.()}>×</button>
      </header>
      <form
        class="form"
        onsubmit={(e) => {
          e.preventDefault()
          submit()
        }}
      >
        <label class="field">
          <span class="label">Filename</span>
          <input
            bind:this={inputEl}
            type="text"
            class="name-input"
            bind:value={nameInput}
            spellcheck="false"
            autocomplete="off"
            onkeydown={onInputKeydown}
          />
        </label>
        <p class="hint">Enter a filename only, e.g. hero.png</p>
        <p class="info">
          References to this asset path are updated automatically across all sites in this project.
          Matching asset files in other sites are renamed too.
        </p>
        <div class="actions">
          <button type="button" class="secondary" onclick={() => onclose?.()}>Cancel</button>
          <button type="submit" class="primary" disabled={!canRename}>Rename</button>
        </div>
      </form>
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
    width: min(26rem, 100%);
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

  .form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .label {
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .name-input {
    box-sizing: border-box;
    width: 100%;
    padding: 0.4rem 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.3rem;
    background: var(--color-bg-root);
    font: inherit;
    font-family: ui-monospace, Consolas, monospace;
    font-size: 0.82rem;
    color: inherit;
  }

  .name-input:focus {
    outline: 2px solid color-mix(in srgb, var(--color-accent) 45%, transparent);
    outline-offset: 1px;
  }

  .hint,
  .info {
    margin: 0;
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }

  .info {
    padding: 0.45rem 0.5rem;
    border-radius: 0.3rem;
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface-1));
    border: 1px solid color-mix(in srgb, var(--color-accent) 22%, var(--color-border-subtle));
    line-height: 1.4;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.4rem;
    margin-top: 0.25rem;
  }

  .actions button {
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.3rem;
    font-size: 0.82rem;
    cursor: pointer;
  }

  .secondary {
    background: transparent;
    color: inherit;
  }

  .secondary:hover {
    background: var(--color-hover);
  }

  .primary {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-bg-root);
    font-weight: 600;
  }

  .primary:hover:not(:disabled) {
    filter: brightness(1.08);
  }

  .primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>

<script lang="ts">
  type Props = {
    props: Record<string, unknown>
    onchange?: (props: Record<string, unknown>) => void
  }

  let { props, onchange }: Props = $props()

  let expanded = $state(false)
  let localText = $state('')
  let parseError = $state('')
  let lastSyncedKey = $state('')

  $effect(() => {
    const key = JSON.stringify(props)
    if (key !== lastSyncedKey) {
      lastSyncedKey = key
      localText = JSON.stringify(props, null, 2)
      parseError = ''
    }
  })

  function commit() {
    try {
      const parsed: unknown = JSON.parse(localText)
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        parseError = 'Props must be a JSON object.'
        return
      }
      parseError = ''
      const next = parsed as Record<string, unknown>
      lastSyncedKey = JSON.stringify(next)
      onchange?.(next)
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      parseError = msg
    }
  }
</script>

<div class="props-json-fallback">
  <button
    type="button"
    class="toggle"
    aria-expanded={expanded}
    onclick={() => (expanded = !expanded)}
  >
    {expanded ? '▼' : '▶'} Props (JSON)
  </button>

  {#if expanded}
    <p class="hint">
      Edit widget <code>props</code> as JSON. Unknown keys are preserved on save.
    </p>
    <textarea
      class="props-textarea"
      rows="10"
      spellcheck="false"
      bind:value={localText}
      onblur={commit}
      onchange={commit}
    ></textarea>
    {#if parseError}
      <p class="parse-error" role="alert">{parseError}</p>
    {/if}
  {/if}
</div>

<style>
  .props-json-fallback {
    margin-top: 0.75rem;
  }

  .toggle {
    padding: 0.25rem 0;
    border: none;
    background: none;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text-primary);
    cursor: pointer;
    text-align: left;
  }

  .toggle:hover {
    color: var(--color-accent, var(--color-text-primary));
  }

  .hint {
    margin: 0.35rem 0 0.5rem;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .props-textarea {
    width: 100%;
    box-sizing: border-box;
    font-family: ui-monospace, Consolas, monospace;
    font-size: 0.78rem;
    resize: vertical;
    min-height: 8rem;
  }

  .parse-error {
    margin: 0.4rem 0 0;
    font-size: 0.78rem;
    color: #e06c75;
    font-family: ui-monospace, Consolas, monospace;
  }
</style>

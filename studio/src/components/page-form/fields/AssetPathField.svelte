<script lang="ts">
  type Props = {
    value: string
    label?: string
    hint?: string
    placeholder?: string
    onchange?: (value: string) => void
  }

  let { value, label, hint, placeholder, onchange }: Props = $props()

  function onInput(event: Event) {
    onchange?.((event.currentTarget as HTMLInputElement).value)
  }
</script>

<div class="asset-path-field">
  {#if label}
    <span class="label">{label}</span>
  {/if}
  {#if hint}
    <span class="hint">{hint}</span>
  {/if}
  <input
    class="path-input"
    type="text"
    {value}
    {placeholder}
    spellcheck="false"
    oninput={onInput}
  />
</div>

<style>
  .asset-path-field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    width: 100%;
  }

  .label {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .hint {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }

  .path-input {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
    font-family: ui-monospace, Consolas, monospace;
  }
</style>

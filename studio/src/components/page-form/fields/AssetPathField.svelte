<script lang="ts">
  import AssetPickerModal from '../../AssetPickerModal.svelte'
  import { getStudioBundleContext } from '../../../lib/studio-bundle-context'

  type Props = {
    value: string
    label?: string
    hint?: string
    placeholder?: string
    onchange?: (value: string) => void
  }

  let { value, label, hint, placeholder, onchange }: Props = $props()

  const bundle = getStudioBundleContext()
  let pickerOpen = $state(false)

  const hasPicker = $derived(!!bundle && bundle.projectRoot.length > 0)

  function onInput(event: Event) {
    onchange?.((event.currentTarget as HTMLInputElement).value)
  }

  function openPicker() {
    if (hasPicker) pickerOpen = true
  }

  function onPickerSelect(path: string) {
    onchange?.(path)
  }
</script>

<div class="asset-path-field">
  {#if label}
    <span class="label">{label}</span>
  {/if}
  {#if hint}
    <span class="hint">{hint}</span>
  {/if}
  <div class="input-wrap">
    <input
      class="path-input"
      type="text"
      {value}
      {placeholder}
      spellcheck="false"
      oninput={onInput}
      onclick={openPicker}
      title={hasPicker ? 'Click to select from assets' : undefined}
    />
    {#if hasPicker}
      <button type="button" class="browse-btn" title="Select asset" onclick={openPicker}>
        …
      </button>
    {/if}
  </div>
</div>

{#if hasPicker && pickerOpen && bundle}
  <AssetPickerModal
    open={pickerOpen}
    projectRoot={bundle.projectRoot}
    sitePath={bundle.sitePath}
    assets={bundle.imageAssets}
    onselect={onPickerSelect}
    onclose={() => (pickerOpen = false)}
  />
{/if}

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

  .input-wrap {
    display: flex;
    gap: 0.25rem;
    align-items: stretch;
  }

  .path-input {
    flex: 1;
    min-width: 0;
    box-sizing: border-box;
    font-size: 0.82rem;
    font-family: ui-monospace, Consolas, monospace;
    cursor: pointer;
  }

  .browse-btn {
    flex-shrink: 0;
    padding: 0 0.45rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.3rem;
    background: var(--color-hover);
    font-size: 0.85rem;
    cursor: pointer;
  }

  .browse-btn:hover {
    background: var(--color-pressed);
  }
</style>

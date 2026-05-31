<script lang="ts">
  import { colorInputValue, showThemeColorPicker } from '../lib/color-value'

  type Props = {
    tokenKey: string
    value: string
    onchange?: (value: string) => void
  }

  let { tokenKey, value, onchange }: Props = $props()

  const pickerVisible = $derived(showThemeColorPicker(tokenKey, value))
  const pickerValue = $derived(colorInputValue(value))

  function onTextInput(event: Event) {
    onchange?.((event.currentTarget as HTMLInputElement).value)
  }

  function onColorInput(event: Event) {
    onchange?.((event.currentTarget as HTMLInputElement).value)
  }
</script>

<div class="color-field">
  {#if pickerVisible}
    <input
      class="color-picker"
      type="color"
      value={pickerValue}
      aria-label="Pick color"
      oninput={onColorInput}
    />
  {/if}
  <input
    class="color-text"
    type="text"
    {value}
    spellcheck="false"
    oninput={onTextInput}
  />
</div>

<style>
  .color-field {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    width: 100%;
  }

  .color-picker {
    width: 2.35rem;
    height: 2.05rem;
    flex-shrink: 0;
    padding: 0.12rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.3rem;
    background: var(--color-surface-1);
    cursor: pointer;
  }

  .color-text {
    flex: 1;
    min-width: 0;
    box-sizing: border-box;
    font-size: 0.82rem;
    font-family: ui-monospace, Consolas, monospace;
  }
</style>

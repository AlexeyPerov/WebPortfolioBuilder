<script lang="ts">
  import { mergeWidgetProps } from '../../../lib/page-form'
  import { readString } from '../../../lib/widget-props'
  import AssetPathField from '../fields/AssetPathField.svelte'

  type Props = {
    props: Record<string, unknown>
    onchange?: (props: Record<string, unknown>) => void
  }

  let { props, onchange }: Props = $props()

  function emit(patch: Record<string, unknown>) {
    onchange?.(mergeWidgetProps(props, patch))
  }
</script>

<div class="widget-props-form">
  <AssetPathField
    label="Source"
    hint="Asset path, e.g. assets/logo.png"
    value={readString(props, 'src')}
    onchange={(src) => emit({ src })}
  />
  <label class="field">
    <span class="label">Alt text</span>
    <input
      type="text"
      value={readString(props, 'alt')}
      oninput={(e) => emit({ alt: e.currentTarget.value })}
    />
  </label>
</div>

<style>
  .widget-props-form {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    margin-top: 0.65rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .label {
    font-weight: 500;
    font-size: 0.82rem;
  }

  input[type='text'] {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
  }
</style>

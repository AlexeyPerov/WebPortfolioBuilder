<script lang="ts">
  import { mergeWidgetProps } from '../../../lib/page-form'
  import { readImageEntries, readString, writeImageEntries } from '../../../lib/widget-props'
  import ImageEntryListField from '../fields/ImageEntryListField.svelte'

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
  <label class="field">
    <span class="label">Aria label</span>
    <input
      type="text"
      value={readString(props, 'aria_label')}
      oninput={(e) => emit({ aria_label: e.currentTarget.value })}
    />
  </label>
  <ImageEntryListField
    label="Slides"
    addLabel="Add slide"
    value={readImageEntries(props.images)}
    onchange={(images) => emit({ images: writeImageEntries(images) })}
  />
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

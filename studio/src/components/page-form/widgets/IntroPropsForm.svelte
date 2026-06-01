<script lang="ts">
  import { mergeWidgetProps } from '../../../lib/page-form'
  import { readString, readStringArray } from '../../../lib/widget-props'
  import StringListField from '../fields/StringListField.svelte'

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
    <span class="label">Title</span>
    <input
      type="text"
      value={readString(props, 'title')}
      oninput={(e) => emit({ title: e.currentTarget.value })}
    />
  </label>
  <StringListField
    label="Paragraphs"
    addLabel="Add paragraph"
    emptyLabel="No paragraphs yet."
    value={readStringArray(props, 'paragraphs')}
    onchange={(paragraphs) => emit({ paragraphs })}
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

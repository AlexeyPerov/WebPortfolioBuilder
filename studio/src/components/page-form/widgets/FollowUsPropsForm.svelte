<script lang="ts">
  import { mergeWidgetProps } from '../../../lib/page-form'
  import { readString } from '../../../lib/widget-props'

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
      value={readString(props, 'title', 'Follow us')}
      oninput={(e) => emit({ title: e.currentTarget.value })}
    />
  </label>
</div>

<style>
  .widget-props-form {
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

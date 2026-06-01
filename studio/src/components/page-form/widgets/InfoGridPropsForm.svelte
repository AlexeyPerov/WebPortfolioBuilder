<script lang="ts">
  import { mergeWidgetProps } from '../../../lib/page-form'
  import {
    readInfoGridItems,
    readString,
    writeInfoGridItems,
    type InfoGridItem,
  } from '../../../lib/widget-props'
  import AssetPathField from '../fields/AssetPathField.svelte'

  type Props = {
    props: Record<string, unknown>
    onchange?: (props: Record<string, unknown>) => void
  }

  let { props, onchange }: Props = $props()

  const items = $derived(readInfoGridItems(props))

  function emit(patch: Record<string, unknown>) {
    onchange?.(mergeWidgetProps(props, patch))
  }

  function updateItems(next: InfoGridItem[]) {
    emit({ items: writeInfoGridItems(next) })
  }

  function updateItem(index: number, patch: Partial<InfoGridItem>) {
    updateItems(items.map((item, i) => (i === index ? { ...item, ...patch } : item)))
  }

  function addItem() {
    updateItems([...items, { image: '', title: '', text: '' }])
  }

  function removeItem(index: number) {
    updateItems(items.filter((_, i) => i !== index))
  }

  function moveItem(index: number, delta: number) {
    const next = [...items]
    const target = index + delta
    if (target < 0 || target >= next.length) return
    ;[next[index], next[target]] = [next[target], next[index]]
    updateItems(next)
  }
</script>

<div class="widget-props-form">
  <label class="field">
    <span class="label">Section title</span>
    <input
      type="text"
      value={readString(props, 'title')}
      oninput={(e) => emit({ title: e.currentTarget.value })}
    />
  </label>

  {#if items.length === 0}
    <p class="empty">No items yet.</p>
  {:else}
    <ul class="item-list">
      {#each items as item, index (index)}
        <li class="item-card">
          <span class="item-head">Item {index + 1}</span>
          <AssetPathField
            label="Image"
            value={item.image}
            onchange={(image) => updateItem(index, { image })}
          />
          <label class="field">
            <span class="label">Title</span>
            <input
              type="text"
              value={item.title}
              oninput={(e) => updateItem(index, { title: e.currentTarget.value })}
            />
          </label>
          <label class="field">
            <span class="label">Text</span>
            <textarea
              rows="2"
              value={item.text}
              oninput={(e) => updateItem(index, { text: e.currentTarget.value })}
            ></textarea>
          </label>
          <div class="row-actions">
            <button type="button" class="icon-btn" disabled={index === 0} onclick={() => moveItem(index, -1)}>↑</button>
            <button type="button" class="icon-btn" disabled={index === items.length - 1} onclick={() => moveItem(index, 1)}>↓</button>
            <button type="button" class="icon-btn danger" onclick={() => removeItem(index)}>×</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <button type="button" class="secondary" onclick={addItem}>Add item</button>
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

  input[type='text'],
  textarea {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
  }

  .empty {
    margin: 0;
    color: var(--color-text-secondary);
    font-style: italic;
    font-size: 0.82rem;
  }

  .item-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .item-card {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    padding: 0.55rem 0.65rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    background: var(--color-surface-1);
  }

  .item-head {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .row-actions {
    display: flex;
    gap: 0.2rem;
  }

  .icon-btn {
    padding: 0.15rem 0.4rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.25rem;
    background: var(--color-surface-1);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .icon-btn.danger {
    color: #e06c75;
  }

  .secondary {
    align-self: flex-start;
    padding: 0.35rem 0.65rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    background: var(--color-hover);
    font-size: 0.82rem;
    cursor: pointer;
  }
</style>

<script lang="ts">
  type Props = {
    value: string[]
    label?: string
    hint?: string
    addLabel?: string
    emptyLabel?: string
    itemPlaceholder?: string
    onchange?: (value: string[]) => void
  }

  let {
    value,
    label,
    hint,
    addLabel = 'Add item',
    emptyLabel = 'No items yet.',
    itemPlaceholder,
    onchange,
  }: Props = $props()

  function newRowKey() {
    return crypto.randomUUID()
  }

  let rowKeys = $state<string[]>([])

  $effect.pre(() => {
    const len = value.length
    if (rowKeys.length < len) {
      rowKeys = [
        ...rowKeys,
        ...Array.from({ length: len - rowKeys.length }, newRowKey),
      ]
    } else if (rowKeys.length > len) {
      rowKeys = rowKeys.slice(0, len)
    }
  })

  function emit(next: string[]) {
    onchange?.(next)
  }

  function updateItem(index: number, text: string) {
    emit(value.map((item, i) => (i === index ? text : item)))
  }

  function addRow() {
    emit([...value, ''])
  }

  function removeRow(index: number) {
    rowKeys = rowKeys.filter((_, i) => i !== index)
    emit(value.filter((_, i) => i !== index))
  }

  function moveRow(index: number, delta: number) {
    const next = [...value]
    const target = index + delta
    if (target < 0 || target >= next.length) return
    ;[next[index], next[target]] = [next[target], next[index]]
    const nextKeys = [...rowKeys]
    ;[nextKeys[index], nextKeys[target]] = [nextKeys[target], nextKeys[index]]
    rowKeys = nextKeys
    emit(next)
  }
</script>

<div class="string-list-field">
  {#if label}
    <span class="label">{label}</span>
  {/if}
  {#if hint}
    <span class="hint">{hint}</span>
  {/if}

  {#if value.length === 0}
    <p class="empty">{emptyLabel}</p>
  {:else}
    <ul class="list">
      {#each value as item, index (rowKeys[index])}
        <li class="list-row">
          <input
            class="item-input"
            type="text"
            value={item}
            placeholder={itemPlaceholder}
            oninput={(e) => updateItem(index, e.currentTarget.value)}
          />
          <div class="row-actions">
            <button
              type="button"
              class="icon-btn"
              title="Move up"
              disabled={index === 0}
              onclick={() => moveRow(index, -1)}>↑</button
            >
            <button
              type="button"
              class="icon-btn"
              title="Move down"
              disabled={index === value.length - 1}
              onclick={() => moveRow(index, 1)}>↓</button
            >
            <button
              type="button"
              class="icon-btn danger"
              title="Remove item"
              onclick={() => removeRow(index)}>×</button
            >
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <button type="button" class="secondary" onclick={addRow}>{addLabel}</button>
</div>

<style>
  .string-list-field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
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

  .empty {
    margin: 0;
    color: var(--color-text-secondary);
    font-style: italic;
    font-size: 0.82rem;
  }

  .list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .list-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .item-input {
    flex: 1;
    min-width: 0;
    box-sizing: border-box;
    font-size: 0.82rem;
    font-family: inherit;
  }

  .row-actions {
    display: flex;
    flex-shrink: 0;
    gap: 0.2rem;
  }

  .icon-btn {
    padding: 0.15rem 0.4rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.25rem;
    background: var(--color-surface-1);
    cursor: pointer;
    font-size: 0.85rem;
    line-height: 1.2;
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--color-hover);
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

  .secondary:hover {
    background: var(--color-pressed);
  }
</style>

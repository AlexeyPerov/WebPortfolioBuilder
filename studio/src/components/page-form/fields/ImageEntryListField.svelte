<script lang="ts">
  import AssetPathField from './AssetPathField.svelte'
  import {
    disableImageEntryAlt,
    enableImageEntryAlt,
    imageEntryAlt,
    imageEntrySrc,
    imageEntryUsesAlt,
    newImageEntry,
    setImageEntryAlt,
    setImageEntrySrc,
    type ImageEntry,
  } from '../../../lib/image-entry'

  type Props = {
    value: ImageEntry[]
    label?: string
    hint?: string
    addLabel?: string
    emptyLabel?: string
    pathHint?: string
    onchange?: (value: ImageEntry[]) => void
  }

  let {
    value,
    label,
    hint,
    addLabel = 'Add image',
    emptyLabel = 'No images yet.',
    pathHint = 'Asset path, e.g. assets/photo.jpg',
    onchange,
  }: Props = $props()

  function emit(next: ImageEntry[]) {
    onchange?.(next)
  }

  function updateEntry(index: number, entry: ImageEntry) {
    emit(value.map((item, i) => (i === index ? entry : item)))
  }

  function addRow() {
    emit([...value, newImageEntry()])
  }

  function removeRow(index: number) {
    emit(value.filter((_, i) => i !== index))
  }

  function moveRow(index: number, delta: number) {
    const next = [...value]
    const target = index + delta
    if (target < 0 || target >= next.length) return
    ;[next[index], next[target]] = [next[target], next[index]]
    emit(next)
  }
</script>

<div class="image-entry-list-field">
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
      {#each value as entry, index (index)}
        <li class="list-row">
          <div class="entry-fields">
            <AssetPathField
              label="Path"
              hint={pathHint}
              value={imageEntrySrc(entry)}
              onchange={(src) => updateEntry(index, setImageEntrySrc(entry, src))}
            />
            {#if imageEntryUsesAlt(entry)}
              <label class="alt-field">
                <span class="alt-label">Alt text</span>
                <input
                  class="alt-input"
                  type="text"
                  value={imageEntryAlt(entry)}
                  oninput={(e) =>
                    updateEntry(index, setImageEntryAlt(entry, e.currentTarget.value))}
                />
              </label>
            {/if}
            <button
              type="button"
              class="text-btn"
              onclick={() =>
                updateEntry(
                  index,
                  imageEntryUsesAlt(entry)
                    ? disableImageEntryAlt(entry)
                    : enableImageEntryAlt(entry),
                )}
            >
              {imageEntryUsesAlt(entry) ? 'Path only' : 'Add alt text'}
            </button>
          </div>
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
              title="Remove image"
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
  .image-entry-list-field {
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
    gap: 0.5rem;
  }

  .list-row {
    display: flex;
    align-items: flex-start;
    gap: 0.4rem;
    padding: 0.5rem 0.55rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    background: var(--color-surface-1);
  }

  .entry-fields {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .alt-field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .alt-label {
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .alt-input {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
  }

  .text-btn {
    align-self: flex-start;
    padding: 0;
    border: none;
    background: none;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    text-decoration: underline;
    cursor: pointer;
  }

  .text-btn:hover {
    color: var(--color-text-primary);
  }

  .row-actions {
    display: flex;
    flex-shrink: 0;
    gap: 0.2rem;
    padding-top: 0.15rem;
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

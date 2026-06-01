<script lang="ts">
  import {
    defaultWidget,
    insertWidget,
    moveWidget,
    removeWidget,
    updateWidget,
    type WidgetNode,
  } from '../../lib/page-form'
  import { WIDGET_TYPE_IDS, widgetTypeLabel } from '../../lib/widget-types'
  import WidgetDetailPanel from './WidgetDetailPanel.svelte'

  type Props = {
    widgets: WidgetNode[]
    selectedIndex: number
    onSelectIndex?: (index: number) => void
    onchange?: (widgets: WidgetNode[]) => void
  }

  let { widgets, selectedIndex, onSelectIndex, onchange }: Props = $props()

  let addType = $state(WIDGET_TYPE_IDS[0] ?? 'intro')

  const clampedIndex = $derived(
    widgets.length === 0 ? -1 : Math.max(0, Math.min(selectedIndex, widgets.length - 1)),
  )

  const selectedWidget = $derived(clampedIndex >= 0 ? widgets[clampedIndex] : null)

  function emitWidgets(next: WidgetNode[]) {
    onchange?.(next)
  }

  function selectIndex(index: number) {
    onSelectIndex?.(index)
  }

  function addWidget() {
    const widget = defaultWidget(addType)
    const next = insertWidget(widgets, widgets.length, widget)
    emitWidgets(next)
    selectIndex(next.length - 1)
  }

  function removeAt(index: number) {
    const next = removeWidget(widgets, index)
    emitWidgets(next)
    if (next.length === 0) {
      selectIndex(0)
    } else if (clampedIndex >= next.length) {
      selectIndex(next.length - 1)
    } else if (index < clampedIndex) {
      selectIndex(clampedIndex - 1)
    }
  }

  function moveAt(index: number, delta: number) {
    const next = moveWidget(widgets, index, delta)
    if (next === widgets) return
    emitWidgets(next)
    const target = index + delta
    if (target >= 0 && target < next.length) {
      selectIndex(target)
    }
  }

  function updateAt(index: number, widget: WidgetNode) {
    emitWidgets(updateWidget(widgets, index, widget))
  }
</script>

<section class="form-section widget-list-section">
  <div class="section-head">
    <h2>Widgets</h2>
    <p class="hint">
      Top-level page blocks. Reorder with ↑↓; edit type, id, enabled, and props for the selected
      widget.
    </p>
  </div>

  {#if widgets.length === 0}
    <p class="empty">No widgets yet. Add one below.</p>
  {:else}
    <ul class="widget-list" role="listbox" aria-label="Page widgets">
      {#each widgets as item, index (index)}
        <li
          class="widget-row"
          class:selected={index === clampedIndex}
          role="option"
          aria-selected={index === clampedIndex}
        >
          <button
            type="button"
            class="widget-select"
            onclick={() => selectIndex(index)}
          >
            <span class="widget-type">{widgetTypeLabel(item.type)}</span>
            {#if item.id}
              <span class="widget-id">{item.id}</span>
            {/if}
            {#if item.enabled === false}
              <span class="widget-badge">disabled</span>
            {/if}
          </button>
          <span class="row-actions">
            <button
              type="button"
              class="icon-btn"
              title="Move up"
              disabled={index === 0}
              onclick={() => moveAt(index, -1)}>↑</button
            >
            <button
              type="button"
              class="icon-btn"
              title="Move down"
              disabled={index === widgets.length - 1}
              onclick={() => moveAt(index, 1)}>↓</button
            >
            <button
              type="button"
              class="icon-btn danger"
              title="Remove widget"
              onclick={() => removeAt(index)}>×</button
            >
          </span>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="add-row">
    <label class="add-type">
      <span class="sr-only">Widget type to add</span>
      <select bind:value={addType}>
        {#each WIDGET_TYPE_IDS as typeId (typeId)}
          <option value={typeId}>{widgetTypeLabel(typeId)}</option>
        {/each}
      </select>
    </label>
    <button type="button" class="secondary" onclick={addWidget}>Add widget</button>
  </div>

  {#if selectedWidget && clampedIndex >= 0}
    <div class="detail-wrap">
      <h3 class="detail-head">Widget {clampedIndex + 1}</h3>
      <WidgetDetailPanel
        widget={selectedWidget}
        onchange={(next) => updateAt(clampedIndex, next)}
      />
    </div>
  {/if}
</section>

<style>
  .widget-list-section .section-head h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .hint {
    margin: 0.25rem 0 0.85rem;
    color: var(--color-text-secondary);
    font-size: 0.8rem;
  }

  .empty {
    margin: 0 0 0.5rem;
    color: var(--color-text-secondary);
    font-style: italic;
  }

  .widget-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    overflow: hidden;
  }

  .widget-row {
    display: flex;
    align-items: stretch;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .widget-row:last-child {
    border-bottom: none;
  }

  .widget-row.selected {
    background: color-mix(in srgb, var(--color-accent, #4a9eff) 12%, var(--color-surface-1));
  }

  .widget-select {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem 0.6rem;
    padding: 0.45rem 0.6rem;
    border: none;
    background: transparent;
    text-align: left;
    font-size: 0.82rem;
    cursor: pointer;
    color: var(--color-text-primary);
  }

  .widget-select:hover {
    background: var(--color-hover);
  }

  .widget-type {
    font-weight: 500;
  }

  .widget-id {
    font-family: ui-monospace, Consolas, monospace;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
  }

  .widget-badge {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--color-text-secondary);
    padding: 0.1rem 0.35rem;
    border-radius: 0.2rem;
    background: var(--color-statusbar-bg);
  }

  .row-actions {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.25rem 0.4rem;
    flex-shrink: 0;
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

  .add-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .add-type select {
    min-width: 12rem;
    font-size: 0.82rem;
  }

  .secondary {
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

  .detail-wrap {
    margin-top: 1rem;
  }

  .detail-head {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>

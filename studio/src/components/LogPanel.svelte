<script lang="ts">
  import BuildLog from './BuildLog.svelte'

  type Props = {
    lines: string[]
    collapsed?: boolean
    heightPx?: number
    onTogglePanel?: () => void
    onheightchange?: (heightPx: number) => void
  }

  let {
    lines,
    collapsed = true,
    heightPx = 112,
    onTogglePanel,
    onheightchange,
  }: Props = $props()

  const COLLAPSED_HEIGHT_PX = 32
  const MIN_EXPANDED_HEIGHT_PX = 64
  const MAX_EXPANDED_HEIGHT_PX = 480

  let dragging = $state(false)

  function startResize(event: PointerEvent) {
    if (collapsed || event.button !== 0) return
    const el = event.currentTarget as HTMLElement
    el.setPointerCapture(event.pointerId)
    dragging = true
    const startY = event.clientY
    const startHeight = heightPx

    const onMove = (e: PointerEvent) => {
      const dy = startY - e.clientY
      const next = Math.min(
        MAX_EXPANDED_HEIGHT_PX,
        Math.max(MIN_EXPANDED_HEIGHT_PX, startHeight + dy),
      )
      onheightchange?.(next)
    }

    const onUp = (e: PointerEvent) => {
      el.releasePointerCapture(e.pointerId)
      el.removeEventListener('pointermove', onMove)
      el.removeEventListener('pointerup', onUp)
      el.removeEventListener('pointercancel', onUp)
      dragging = false
    }

    el.addEventListener('pointermove', onMove)
    el.addEventListener('pointerup', onUp)
    el.addEventListener('pointercancel', onUp)
    event.preventDefault()
  }

  const panelHeight = $derived(collapsed ? COLLAPSED_HEIGHT_PX : heightPx)
</script>

<footer
  class="log-panel"
  class:collapsed
  class:dragging
  style:height="{panelHeight}px"
  style:min-height="{panelHeight}px"
>
  {#if !collapsed}
    <button
      type="button"
      class="resize-handle"
      aria-label="Resize logs panel"
      onpointerdown={startResize}
    ></button>
  {/if}
  <div class="log-body">
    <BuildLog {lines} {collapsed} {onTogglePanel} />
  </div>
</footer>

<style>
  .log-panel {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    position: relative;
    border-top: 1px solid var(--color-border-subtle);
    background: var(--color-surface-1);
    overflow: hidden;
  }

  .log-panel.collapsed .log-body {
    overflow: hidden;
  }

  .log-panel.dragging {
    cursor: row-resize;
    user-select: none;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 5px;
    padding: 0;
    border: none;
    background: transparent;
    cursor: row-resize;
    z-index: 1;
    touch-action: none;
  }

  .resize-handle:hover,
  .resize-handle:focus-visible {
    background: color-mix(in srgb, var(--color-accent) 35%, transparent);
  }

  .log-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>

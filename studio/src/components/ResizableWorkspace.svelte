<script lang="ts">
  import type { Snippet } from 'svelte'
  import { untrack } from 'svelte'
  import {
    GUTTER_WIDTH_PX,
    layoutFromSettings,
    normalizeLayout,
    type WorkspaceLayout,
  } from '../lib/workspace-layout'

  type Props = {
    sidebarWidth: number
    previewWidth: number
    savedLayout?: WorkspaceLayout | null
    sidebar: Snippet
    main: Snippet
    preview: Snippet
    onlayoutcommit?: (sidebar: number, preview: number) => void
  }

  let {
    sidebarWidth = $bindable(),
    previewWidth = $bindable(),
    savedLayout = null,
    sidebar,
    main,
    preview,
    onlayoutcommit,
  }: Props = $props()

  let root = $state<HTMLDivElement | null>(null)
  let dragging = $state(false)
  let savedLayoutApplied = $state(false)

  function containerWidth(): number {
    return root?.clientWidth ?? 0
  }

  function applySavedLayout() {
    if (savedLayoutApplied || !savedLayout) return false
    const w = containerWidth()
    if (w <= 0) return false
    const next = layoutFromSettings(w, savedLayout)
    sidebarWidth = next.sidebar
    previewWidth = next.preview
    savedLayoutApplied = true
    return true
  }

  function applyNormalized() {
    untrack(() => {
      if (applySavedLayout()) return
      const w = containerWidth()
      if (w <= 0) return
      const next = normalizeLayout(w, sidebarWidth, previewWidth)
      sidebarWidth = next.sidebar
      previewWidth = next.preview
    })
  }

  function startDrag(which: 'sidebar' | 'preview', event: PointerEvent) {
    if (event.button !== 0) return
    const el = event.currentTarget as HTMLElement
    el.setPointerCapture(event.pointerId)
    dragging = true
    const startX = event.clientX
    const startSidebar = sidebarWidth
    const startPreview = previewWidth

    const onMove = (e: PointerEvent) => {
      const w = containerWidth()
      if (w <= 0) return
      const dx = e.clientX - startX
      if (which === 'sidebar') {
        const next = normalizeLayout(w, startSidebar + dx, previewWidth)
        sidebarWidth = next.sidebar
        previewWidth = next.preview
      } else {
        const next = normalizeLayout(w, sidebarWidth, startPreview - dx)
        sidebarWidth = next.sidebar
        previewWidth = next.preview
      }
    }

    const onUp = (e: PointerEvent) => {
      el.releasePointerCapture(e.pointerId)
      el.removeEventListener('pointermove', onMove)
      el.removeEventListener('pointerup', onUp)
      el.removeEventListener('pointercancel', onUp)
      dragging = false
      applyNormalized()
      onlayoutcommit?.(sidebarWidth, previewWidth)
    }

    el.addEventListener('pointermove', onMove)
    el.addEventListener('pointerup', onUp)
    el.addEventListener('pointercancel', onUp)
    event.preventDefault()
  }

  function onWindowResize() {
    applyNormalized()
  }

  $effect(() => {
    savedLayout
    savedLayoutApplied = false
    applyNormalized()
  })

  $effect(() => {
    const el = root
    if (!el) return
    applyNormalized()
    const ro = new ResizeObserver(() => applyNormalized())
    ro.observe(el)
    window.addEventListener('resize', onWindowResize)
    return () => {
      ro.disconnect()
      window.removeEventListener('resize', onWindowResize)
    }
  })
</script>

<div class="workspace" class:dragging bind:this={root}>
  <aside class="pane sidebar" style:width="{sidebarWidth}px">
    {@render sidebar()}
  </aside>

  <button
    type="button"
    class="gutter"
    aria-label="Resize file tree"
    style:width="{GUTTER_WIDTH_PX}px"
    onpointerdown={(e) => startDrag('sidebar', e)}
  ></button>

  <section class="pane center">
    {@render main()}
  </section>

  <button
    type="button"
    class="gutter"
    aria-label="Resize preview"
    style:width="{GUTTER_WIDTH_PX}px"
    onpointerdown={(e) => startDrag('preview', e)}
  ></button>

  <aside class="pane preview-pane" style:width="{previewWidth}px">
    {@render preview()}
  </aside>
</div>

<style>
  .workspace {
    display: flex;
    flex: 1;
    min-height: 0;
    min-width: 0;
  }

  .workspace.dragging {
    cursor: col-resize;
    user-select: none;
  }

  .workspace.dragging :global(iframe) {
    pointer-events: none;
  }

  .pane {
    min-height: 0;
    overflow: hidden;
  }

  .pane.sidebar {
    flex-shrink: 0;
    border-right: none;
  }

  .pane.center {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .pane.preview-pane {
    flex-shrink: 0;
  }

  .gutter {
    flex-shrink: 0;
    margin: 0;
    padding: 0;
    border: none;
    cursor: col-resize;
    background: var(--color-border-subtle);
    transition: background 0.12s ease;
    touch-action: none;
  }

  .gutter:hover,
  .gutter:focus-visible {
    background: color-mix(in srgb, var(--color-accent) 45%, var(--color-border-subtle));
  }

  .gutter:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -1px;
  }
</style>

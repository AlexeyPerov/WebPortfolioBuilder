/** Horizontal split layout for the studio workspace (px). */

export const GUTTER_WIDTH_PX = 5
export const MIN_SIDEBAR_PX = 160
export const MIN_PREVIEW_PX = 280
export const MIN_CENTER_PX = 320
export const DEFAULT_SIDEBAR_PX = 192
export const DEFAULT_PREVIEW_RATIO = 0.38

export type WorkspaceLayout = {
  sidebar_px?: number | null
  preview_px?: number | null
}

export function defaultPreviewWidth(containerWidth: number): number {
  const gutters = 2 * GUTTER_WIDTH_PX
  const raw = Math.round(containerWidth * DEFAULT_PREVIEW_RATIO)
  const maxPreview =
    containerWidth - MIN_CENTER_PX - DEFAULT_SIDEBAR_PX - gutters
  return Math.max(MIN_PREVIEW_PX, Math.min(raw, maxPreview))
}

export function clampSidebarWidth(
  containerWidth: number,
  sidebar: number,
  previewWidth: number,
): number {
  const gutters = 2 * GUTTER_WIDTH_PX
  const max =
    containerWidth - MIN_CENTER_PX - MIN_PREVIEW_PX - previewWidth - gutters
  return Math.round(Math.max(MIN_SIDEBAR_PX, Math.min(sidebar, max)))
}

export function clampPreviewWidth(
  containerWidth: number,
  preview: number,
  sidebarWidth: number,
): number {
  const gutters = 2 * GUTTER_WIDTH_PX
  const max =
    containerWidth - MIN_CENTER_PX - MIN_SIDEBAR_PX - sidebarWidth - gutters
  return Math.round(Math.max(MIN_PREVIEW_PX, Math.min(preview, max)))
}

export function layoutFromSettings(
  containerWidth: number,
  saved?: WorkspaceLayout | null,
): { sidebar: number; preview: number } {
  const previewDefault = defaultPreviewWidth(containerWidth)
  const sidebarRaw = saved?.sidebar_px ?? DEFAULT_SIDEBAR_PX
  const previewRaw = saved?.preview_px ?? previewDefault
  const preview = clampPreviewWidth(containerWidth, previewRaw, sidebarRaw)
  const sidebar = clampSidebarWidth(containerWidth, sidebarRaw, preview)
  return { sidebar, preview }
}

export function normalizeLayout(
  containerWidth: number,
  sidebar: number,
  preview: number,
): { sidebar: number; preview: number } {
  const previewClamped = clampPreviewWidth(containerWidth, preview, sidebar)
  const sidebarClamped = clampSidebarWidth(
    containerWidth,
    sidebar,
    previewClamped,
  )
  return { sidebar: sidebarClamped, preview: previewClamped }
}

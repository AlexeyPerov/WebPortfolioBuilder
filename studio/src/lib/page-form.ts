export type PageSeo = {
  description: string
  og_image: string
  canonical_url: string
}

export type PageLayout = {
  hide_header: boolean
  hide_footer: boolean
}

export type WidgetNode = {
  type: string
  id?: string
  enabled?: boolean
  props: Record<string, unknown>
}

export type PageFormModel = {
  slug: string
  title: string
  seo: PageSeo
  layout: PageLayout
  widgets: WidgetNode[]
}

export type ParsePageFormResult =
  | { ok: true; model: PageFormModel; doc: Record<string, unknown> }
  | { ok: false; error: string }

function readSeo(raw: unknown): PageSeo {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    return { description: '', og_image: '', canonical_url: '' }
  }
  const row = raw as Record<string, unknown>
  return {
    description: typeof row.description === 'string' ? row.description : '',
    og_image: typeof row.og_image === 'string' ? row.og_image : '',
    canonical_url: typeof row.canonical_url === 'string' ? row.canonical_url : '',
  }
}

function readLayout(raw: unknown): PageLayout {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    return { hide_header: false, hide_footer: false }
  }
  const row = raw as Record<string, unknown>
  return {
    hide_header: row.hide_header === true,
    hide_footer: row.hide_footer === true,
  }
}

function readProps(raw: unknown): Record<string, unknown> {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return {}
  return { ...(raw as Record<string, unknown>) }
}

function readWidgetNode(raw: unknown): WidgetNode {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    return { type: 'intro', props: {} }
  }
  const row = raw as Record<string, unknown>
  const widget: WidgetNode = {
    type: typeof row.type === 'string' ? row.type : 'intro',
    props: readProps(row.props),
  }
  if (typeof row.id === 'string' && row.id.trim() !== '') {
    widget.id = row.id
  }
  if (row.enabled === false) {
    widget.enabled = false
  }
  return widget
}

function readWidgets(raw: unknown): WidgetNode[] {
  if (!Array.isArray(raw)) return []
  return raw.map(readWidgetNode)
}

function writeWidgetNode(node: WidgetNode): Record<string, unknown> {
  const out: Record<string, unknown> = { type: node.type }
  if (node.id?.trim()) out.id = node.id.trim()
  if (node.enabled === false) out.enabled = false
  if (Object.keys(node.props).length > 0) {
    out.props = node.props
  }
  return out
}

/** Spread original props first so additionalProperties keys are preserved. */
export function mergeWidgetProps(
  existing: Record<string, unknown>,
  patch: Record<string, unknown>,
): Record<string, unknown> {
  return { ...existing, ...patch }
}

export { defaultPropsForType, defaultWidget } from './widget-types'

export function insertWidget(
  widgets: WidgetNode[],
  index: number,
  widget: WidgetNode,
): WidgetNode[] {
  const next = [...widgets]
  const at = Math.max(0, Math.min(index, next.length))
  next.splice(at, 0, widget)
  return next
}

export function removeWidget(widgets: WidgetNode[], index: number): WidgetNode[] {
  return widgets.filter((_, i) => i !== index)
}

export function moveWidget(widgets: WidgetNode[], index: number, delta: number): WidgetNode[] {
  const target = index + delta
  if (target < 0 || target >= widgets.length || target === index) return widgets
  const next = [...widgets]
  ;[next[index], next[target]] = [next[target], next[index]]
  return next
}

export function updateWidget(
  widgets: WidgetNode[],
  index: number,
  patch: Partial<WidgetNode>,
): WidgetNode[] {
  return widgets.map((widget, i) => {
    if (i !== index) return widget
    const next: WidgetNode = { ...widget, ...patch }
    if (patch.props !== undefined) {
      next.props = patch.props
    }
    return next
  })
}

export function updateWidgetProps(
  widgets: WidgetNode[],
  index: number,
  propsPatch: Record<string, unknown>,
): WidgetNode[] {
  const widget = widgets[index]
  if (!widget) return widgets
  return updateWidget(widgets, index, {
    props: mergeWidgetProps(widget.props, propsPatch),
  })
}

export function parsePageForm(text: string): ParsePageFormResult {
  let doc: unknown
  try {
    doc = JSON.parse(text)
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    return { ok: false, error: msg }
  }

  if (!doc || typeof doc !== 'object' || Array.isArray(doc)) {
    return { ok: false, error: 'Page JSON must be a JSON object.' }
  }

  const record = doc as Record<string, unknown>
  return {
    ok: true,
    doc: record,
    model: {
      slug: typeof record.slug === 'string' ? record.slug : '',
      title: typeof record.title === 'string' ? record.title : '',
      seo: readSeo(record.seo),
      layout: readLayout(record.layout),
      widgets: readWidgets(record.widgets),
    },
  }
}

export function applyPageForm(doc: Record<string, unknown>, model: PageFormModel): string {
  const next: Record<string, unknown> = { ...doc }

  next.slug = model.slug

  if (model.title.trim()) {
    next.title = model.title
  } else {
    delete next.title
  }

  const seo: Record<string, string> = {}
  if (model.seo.description.trim()) seo.description = model.seo.description
  if (model.seo.og_image.trim()) seo.og_image = model.seo.og_image
  if (model.seo.canonical_url.trim()) seo.canonical_url = model.seo.canonical_url
  if (Object.keys(seo).length > 0) {
    next.seo = seo
  } else {
    delete next.seo
  }

  if (model.layout.hide_header || model.layout.hide_footer) {
    const layout: Record<string, boolean> = {}
    if (model.layout.hide_header) layout.hide_header = true
    if (model.layout.hide_footer) layout.hide_footer = true
    next.layout = layout
  } else {
    delete next.layout
  }

  next.widgets = model.widgets.map(writeWidgetNode)

  return `${JSON.stringify(next, null, 2)}\n`
}

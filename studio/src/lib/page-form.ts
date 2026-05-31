export type PageSeo = {
  description: string
  og_image: string
  canonical_url: string
}

export type PageLayout = {
  hide_header: boolean
  hide_footer: boolean
}

export type PageFormModel = {
  slug: string
  title: string
  seo: PageSeo
  layout: PageLayout
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

  return `${JSON.stringify(next, null, 2)}\n`
}

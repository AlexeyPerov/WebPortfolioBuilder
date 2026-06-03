import siteSchema from '@schema/site.schema.json'

export type NavItem = {
  label: string
  href: string
  open_in_new_tab: boolean
}

export type SiteFormModel = {
  theme: Record<string, string>
  nav: NavItem[]
}

export type ParseSiteFormResult =
  | { ok: true; model: SiteFormModel; doc: Record<string, unknown> }
  | { ok: false; error: string }

const themeProperties =
  (
    siteSchema as {
      properties: { theme: { properties: Record<string, unknown> } }
    }
  ).properties.theme.properties ?? {}

/** Theme keys from site.schema.json — form shows these plus any extra keys in the file. */
export const KNOWN_THEME_TOKENS = Object.keys(themeProperties)

const THEME_LABELS: Record<string, string> = {
  page_bg: 'Page background',
  surface: 'Surface',
  surface_strong: 'Surface (strong)',
  text_primary: 'Text (primary)',
  text_secondary: 'Text (secondary)',
  accent: 'Accent',
  accent_muted: 'Accent (muted)',
  line: 'Line / border',
  widget_gradient: 'Widget gradient',
  cover_background: 'Cover background',
  intro_gradient: 'Intro section gradient',
  catalog_gradient: 'Catalog section gradient',
  project_grid_gradient: 'Project grid section gradient',
  offers_gradient: 'Offers section gradient',
  photos_gradient: 'Photos section gradient',
  vacancies_gradient: 'Vacancies section gradient',
  follow_us_gradient: 'Follow us section gradient',
  footer_gradient: 'Footer section gradient',
  social_button_background: 'Social icon tile background',
  social_icon_github: 'Social icon — GitHub',
  social_icon_linkedin: 'Social icon — LinkedIn',
  social_icon_facebook: 'Social icon — Facebook',
}

export function themeTokenLabel(key: string): string {
  return THEME_LABELS[key] ?? key.replace(/_/g, ' ')
}

function emptyNavItem(): NavItem {
  return { label: '', href: '', open_in_new_tab: false }
}

function readTheme(raw: unknown): Record<string, string> {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return {}
  const theme: Record<string, string> = {}
  for (const [key, value] of Object.entries(raw as Record<string, unknown>)) {
    if (typeof value === 'string') theme[key] = value
  }
  return theme
}

function readNav(raw: unknown): NavItem[] {
  if (!Array.isArray(raw)) return []
  return raw.map((item) => {
    if (!item || typeof item !== 'object' || Array.isArray(item)) {
      return emptyNavItem()
    }
    const row = item as Record<string, unknown>
    return {
      label: typeof row.label === 'string' ? row.label : '',
      href: typeof row.href === 'string' ? row.href : '',
      open_in_new_tab: row.open_in_new_tab === true,
    }
  })
}

export function parseSiteForm(text: string): ParseSiteFormResult {
  let doc: unknown
  try {
    doc = JSON.parse(text)
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    return { ok: false, error: msg }
  }

  if (!doc || typeof doc !== 'object' || Array.isArray(doc)) {
    return { ok: false, error: 'site.json must be a JSON object.' }
  }

  const record = doc as Record<string, unknown>
  const header =
    record.header && typeof record.header === 'object' && !Array.isArray(record.header)
      ? (record.header as Record<string, unknown>)
      : {}

  return {
    ok: true,
    doc: record,
    model: {
      theme: readTheme(record.theme),
      nav: readNav(header.nav),
    },
  }
}

export function applySiteForm(doc: Record<string, unknown>, model: SiteFormModel): string {
  const next: Record<string, unknown> = { ...doc }

  const theme: Record<string, string> = {}
  for (const [key, value] of Object.entries(model.theme)) {
    if (value.trim() !== '') theme[key] = value
  }
  next.theme = theme

  const header =
    next.header && typeof next.header === 'object' && !Array.isArray(next.header)
      ? { ...(next.header as Record<string, unknown>) }
      : {}

  header.nav = model.nav.map((item) => ({
    label: item.label,
    href: item.href,
    open_in_new_tab: item.open_in_new_tab,
  }))
  next.header = header

  return `${JSON.stringify(next, null, 2)}\n`
}

export function mergeThemeForEdit(
  theme: Record<string, string>,
): { known: { key: string; value: string }[]; extra: { key: string; value: string }[] } {
  const knownKeys = new Set(KNOWN_THEME_TOKENS)
  const known = KNOWN_THEME_TOKENS.map((key) => ({
    key,
    value: theme[key] ?? '',
  }))
  const extra = Object.entries(theme)
    .filter(([key]) => !knownKeys.has(key))
    .map(([key, value]) => ({ key, value }))
    .sort((a, b) => a.key.localeCompare(b.key))
  return { known, extra }
}

export function newEmptyNavItem(): NavItem {
  return emptyNavItem()
}

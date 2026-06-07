import pageSchema from '@schema/page.schema.json'

type PageSchemaWithWidgetType = {
  $defs: {
    widgetType: { enum: string[] }
  }
}

/** Closed v1 widget registry from page.schema.json. */
export const WIDGET_TYPE_IDS = (pageSchema as PageSchemaWithWidgetType).$defs.widgetType
  .enum as readonly string[]

export type WidgetTypeId = (typeof WIDGET_TYPE_IDS)[number]

const WIDGET_LABELS: Record<string, string> = {
  intro: 'Intro',
  apps_showcase: 'Apps showcase',
  info_grid: 'Info grid',
  images_grid: 'Images grid',
  careers_tabs: 'Careers tabs',
  follow_us: 'Follow us',
  cover_banner: 'Cover banner',
  project_grid: 'Project grid',
  media_swiper: 'Media swiper',
  reference_panel: 'Reference panel',
  row: 'Row',
  column: 'Column',
  grid: 'Grid',
}

export function widgetTypeLabel(type: string): string {
  return WIDGET_LABELS[type] ?? type.replace(/_/g, ' ')
}

/** Widget types with dedicated Form-tab prop editors (Phases 1–3). */
export const SUPPORTED_FORM_WIDGET_TYPES = [
  'cover_banner',
  'intro',
  'follow_us',
  'info_grid',
  'images_grid',
  'media_swiper',
  'apps_showcase',
  'careers_tabs',
  'project_grid',
] as const

export type SupportedFormWidgetType = (typeof SUPPORTED_FORM_WIDGET_TYPES)[number]

export function isSupportedFormWidget(type: string): type is SupportedFormWidgetType {
  return (SUPPORTED_FORM_WIDGET_TYPES as readonly string[]).includes(type)
}

const LAYOUT_WIDGET_TYPES = new Set(['row', 'column', 'grid'])

export function isLayoutWidget(type: string): boolean {
  return LAYOUT_WIDGET_TYPES.has(type)
}

export function defaultPropsForType(type: string): Record<string, unknown> {
  switch (type) {
    case 'cover_banner':
      return { src: '', alt: '' }
    case 'intro':
      return { title: '', paragraphs: [] }
    case 'follow_us':
      return { title: 'Follow us' }
    case 'info_grid':
      return {
        title: '',
        items: [{ image: '', title: '', text: '' }],
      }
    case 'images_grid':
      return {
        title: '',
        images: [{ src: '', alt: '' }],
      }
    case 'media_swiper':
      return {
        aria_label: '',
        images: [{ src: '', alt: '' }],
      }
    case 'careers_tabs':
      return {
        title: '',
        vacancies: [{ role: '', requirements: [], responsibilities: [], advantages: [] }],
      }
    case 'apps_showcase':
      return {
        section_title: '',
        apps: [{ image: '' }],
      }
    case 'project_grid':
      return {
        heading: '',
        subheading: '',
        cards: [
          {
            title: '',
            description: '',
            tags: [],
            meta: '',
            cta: { label: '', url: '' },
          },
        ],
      }
    case 'row':
    case 'column':
    case 'grid':
      return {
        children: [{ type: 'intro', props: { title: '', paragraphs: [] } }],
      }
    default:
      return {}
  }
}

/** Minimal valid widget node for "Add widget" in the page form. */
export function defaultWidget(type: string): {
  type: string
  props: Record<string, unknown>
} {
  const resolved =
    WIDGET_TYPE_IDS.includes(type) ? type : (WIDGET_TYPE_IDS[0] ?? 'intro')
  return { type: resolved, props: defaultPropsForType(resolved) }
}

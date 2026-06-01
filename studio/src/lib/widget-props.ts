import { readImageEntries, type ImageEntry } from './image-entry'

export function readString(
  props: Record<string, unknown>,
  key: string,
  fallback = '',
): string {
  const value = props[key]
  return typeof value === 'string' ? value : fallback
}

export function readStringArray(props: Record<string, unknown>, key: string): string[] {
  const value = props[key]
  if (!Array.isArray(value)) return []
  return value.map((item) => (typeof item === 'string' ? item : ''))
}

export function readRecordArray(
  props: Record<string, unknown>,
  key: string,
): Record<string, unknown>[] {
  const value = props[key]
  if (!Array.isArray(value)) return []
  return value.filter(
    (item): item is Record<string, unknown> =>
      !!item && typeof item === 'object' && !Array.isArray(item),
  )
}

export type InfoGridItem = {
  image: string
  title: string
  text: string
}

export function readInfoGridItems(props: Record<string, unknown>): InfoGridItem[] {
  return readRecordArray(props, 'items').map((row) => ({
    image: readString(row, 'image'),
    title: readString(row, 'title'),
    text: readString(row, 'text'),
  }))
}

export function writeInfoGridItems(items: InfoGridItem[]): Record<string, unknown>[] {
  return items.map((item) => ({
    image: item.image,
    title: item.title,
    text: item.text,
  }))
}

export type CareersLabels = {
  requirements_title: string
  responsibilities_title: string
  advantages_title: string
}

export function readCareersLabels(props: Record<string, unknown>): CareersLabels {
  const raw = props.labels
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    return { requirements_title: '', responsibilities_title: '', advantages_title: '' }
  }
  const row = raw as Record<string, unknown>
  return {
    requirements_title: readString(row, 'requirements_title'),
    responsibilities_title: readString(row, 'responsibilities_title'),
    advantages_title: readString(row, 'advantages_title'),
  }
}

export function writeCareersLabels(labels: CareersLabels): Record<string, string> | undefined {
  const out: Record<string, string> = {}
  if (labels.requirements_title.trim()) out.requirements_title = labels.requirements_title
  if (labels.responsibilities_title.trim()) {
    out.responsibilities_title = labels.responsibilities_title
  }
  if (labels.advantages_title.trim()) out.advantages_title = labels.advantages_title
  return Object.keys(out).length > 0 ? out : undefined
}

export type VacancyItem = {
  role: string
  requirements: string[]
  responsibilities: string[]
  advantages: string[]
  apply_url: string
  apply_label: string
}

export function readVacancies(props: Record<string, unknown>): VacancyItem[] {
  return readRecordArray(props, 'vacancies').map((row) => ({
    role: readString(row, 'role'),
    requirements: readStringArray(row, 'requirements'),
    responsibilities: readStringArray(row, 'responsibilities'),
    advantages: readStringArray(row, 'advantages'),
    apply_url: readString(row, 'apply_url'),
    apply_label: readString(row, 'apply_label'),
  }))
}

export function writeVacancies(vacancies: VacancyItem[]): Record<string, unknown>[] {
  return vacancies.map((v) => {
    const row: Record<string, unknown> = { role: v.role }
    if (v.requirements.length > 0) row.requirements = v.requirements
    if (v.responsibilities.length > 0) row.responsibilities = v.responsibilities
    if (v.advantages.length > 0) row.advantages = v.advantages
    if (v.apply_url.trim()) row.apply_url = v.apply_url
    if (v.apply_label.trim()) row.apply_label = v.apply_label
    return row
  })
}

export type CatalogApp = {
  image: string
  header_image: string
  swiper_images: string[]
  card_background: string
  title: string
  text_1: string
  text_2: string
  stat_left_line_1: string
  stat_left_line_2: string
  stat_right_line_1: string
  stat_right_line_2: string
  google_play_url: string
  app_store_url: string
  amazon_store_url: string
  galaxy_store_url: string
}

export function readCatalogApps(props: Record<string, unknown>): CatalogApp[] {
  return readRecordArray(props, 'apps').map((row) => ({
    image: readString(row, 'image'),
    header_image: readString(row, 'header_image'),
    swiper_images: readStringArray(row, 'swiper_images'),
    card_background: readString(row, 'card_background'),
    title: readString(row, 'title'),
    text_1: readString(row, 'text_1'),
    text_2: readString(row, 'text_2'),
    stat_left_line_1: readString(row, 'stat_left_line_1'),
    stat_left_line_2: readString(row, 'stat_left_line_2'),
    stat_right_line_1: readString(row, 'stat_right_line_1'),
    stat_right_line_2: readString(row, 'stat_right_line_2'),
    google_play_url: readString(row, 'google_play_url'),
    app_store_url: readString(row, 'app_store_url'),
    amazon_store_url: readString(row, 'amazon_store_url'),
    galaxy_store_url: readString(row, 'galaxy_store_url'),
  }))
}

export function writeCatalogApp(app: CatalogApp): Record<string, unknown> {
  const row: Record<string, unknown> = { image: app.image }
  if (app.header_image.trim()) row.header_image = app.header_image
  if (app.swiper_images.length > 0) row.swiper_images = app.swiper_images
  if (app.card_background.trim()) row.card_background = app.card_background
  if (app.title.trim()) row.title = app.title
  if (app.text_1.trim()) row.text_1 = app.text_1
  if (app.text_2.trim()) row.text_2 = app.text_2
  if (app.stat_left_line_1.trim()) row.stat_left_line_1 = app.stat_left_line_1
  if (app.stat_left_line_2.trim()) row.stat_left_line_2 = app.stat_left_line_2
  if (app.stat_right_line_1.trim()) row.stat_right_line_1 = app.stat_right_line_1
  if (app.stat_right_line_2.trim()) row.stat_right_line_2 = app.stat_right_line_2
  if (app.google_play_url.trim()) row.google_play_url = app.google_play_url
  if (app.app_store_url.trim()) row.app_store_url = app.app_store_url
  if (app.amazon_store_url.trim()) row.amazon_store_url = app.amazon_store_url
  if (app.galaxy_store_url.trim()) row.galaxy_store_url = app.galaxy_store_url
  return row
}

export type ProjectGridCta = {
  label: string
  url: string
}

export type ProjectGridCard = {
  title: string
  description: string
  tags: string[]
  image: string
  metaMode: 'string' | 'object'
  metaString: string
  metaPairs: { key: string; value: string }[]
  cta: ProjectGridCta
}

function readMetaPairs(raw: unknown): { key: string; value: string }[] {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return []
  return Object.entries(raw as Record<string, unknown>)
    .filter(([, v]) => typeof v === 'string')
    .map(([key, value]) => ({ key, value: value as string }))
}

export function readProjectGridCards(props: Record<string, unknown>): ProjectGridCard[] {
  return readRecordArray(props, 'cards').map((row) => {
    const meta = row.meta
    if (typeof meta === 'string') {
      return {
        title: readString(row, 'title'),
        description: readString(row, 'description'),
        tags: readStringArray(row, 'tags'),
        image: readString(row, 'image'),
        metaMode: 'string' as const,
        metaString: meta,
        metaPairs: [],
        cta: readProjectGridCta(row.cta),
      }
    }
    return {
      title: readString(row, 'title'),
      description: readString(row, 'description'),
      tags: readStringArray(row, 'tags'),
      image: readString(row, 'image'),
      metaMode: 'object' as const,
      metaString: '',
      metaPairs: readMetaPairs(meta),
      cta: readProjectGridCta(row.cta),
    }
  })
}

function readProjectGridCta(raw: unknown): ProjectGridCta {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    return { label: '', url: '' }
  }
  const row = raw as Record<string, unknown>
  return {
    label: readString(row, 'label'),
    url: readString(row, 'url'),
  }
}

export function writeProjectGridCard(card: ProjectGridCard): Record<string, unknown> {
  const row: Record<string, unknown> = {
    title: card.title,
    description: card.description,
    tags: card.tags,
    cta: {
      url: card.cta.url,
      ...(card.cta.label.trim() ? { label: card.cta.label } : {}),
    },
  }
  if (card.image.trim()) row.image = card.image
  if (card.metaMode === 'string') {
    if (card.metaString.trim()) row.meta = card.metaString
  } else {
    const meta: Record<string, string> = {}
    for (const pair of card.metaPairs) {
      if (pair.key.trim()) meta[pair.key.trim()] = pair.value
    }
    if (Object.keys(meta).length > 0) row.meta = meta
  }
  return row
}

export { readImageEntries }

export function writeImageEntries(entries: ImageEntry[]): unknown[] {
  return entries.map((entry) => {
    if (typeof entry === 'string') return entry
    if (!entry.alt?.trim()) return entry.src
    return { src: entry.src, alt: entry.alt }
  })
}

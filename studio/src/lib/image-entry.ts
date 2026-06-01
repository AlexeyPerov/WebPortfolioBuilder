/** Page schema image list item: path string or `{ src, alt? }`. */
export type ImageEntry = string | { src: string; alt?: string }

export function imageEntrySrc(entry: ImageEntry): string {
  return typeof entry === 'string' ? entry : entry.src
}

export function imageEntryAlt(entry: ImageEntry): string {
  return typeof entry === 'string' ? '' : (entry.alt ?? '')
}

export function imageEntryUsesAlt(entry: ImageEntry): boolean {
  return typeof entry !== 'string'
}

export function setImageEntrySrc(entry: ImageEntry, src: string): ImageEntry {
  if (typeof entry === 'string') return src
  return { ...entry, src }
}

export function setImageEntryAlt(entry: ImageEntry, alt: string): ImageEntry {
  const src = imageEntrySrc(entry)
  if (!alt.trim()) return src
  return { src, alt }
}

export function enableImageEntryAlt(entry: ImageEntry): ImageEntry {
  return { src: imageEntrySrc(entry), alt: imageEntryAlt(entry) }
}

export function disableImageEntryAlt(entry: ImageEntry): ImageEntry {
  return imageEntrySrc(entry)
}

export function newImageEntry(path = ''): ImageEntry {
  return path
}

export function readImageEntries(raw: unknown): ImageEntry[] {
  if (!Array.isArray(raw)) return []
  const entries: ImageEntry[] = []
  for (const item of raw) {
    if (typeof item === 'string') {
      entries.push(item)
    } else if (item && typeof item === 'object' && !Array.isArray(item)) {
      const row = item as Record<string, unknown>
      if (typeof row.src === 'string') {
        entries.push(
          typeof row.alt === 'string' && row.alt.trim()
            ? { src: row.src, alt: row.alt }
            : row.src,
        )
      }
    }
  }
  return entries
}

const IMAGE_EXTENSIONS = new Set([
  'png',
  'jpg',
  'jpeg',
  'gif',
  'webp',
  'svg',
  'ico',
  'avif',
])

export function isImageFile(relativePath: string): boolean {
  const name = relativePath.replace(/\\/g, '/').split('/').pop() ?? ''
  const dot = name.lastIndexOf('.')
  if (dot < 0) return false
  return IMAGE_EXTENSIONS.has(name.slice(dot + 1).toLowerCase())
}

export function isJsonFile(relativePath: string): boolean {
  const rel = relativePath.replace(/\\/g, '/')
  return rel === 'site.json' || (rel.startsWith('pages/') && rel.endsWith('.json'))
}

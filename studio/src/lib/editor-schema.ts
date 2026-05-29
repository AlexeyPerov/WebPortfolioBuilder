import Ajv from 'ajv'
import siteSchema from '@schema/site.schema.json'
import pageSchema from '@schema/page.schema.json'

const ajv = new Ajv({ allErrors: true, strict: false })

const validateSiteDoc = ajv.compile(siteSchema)
const validatePageDoc = ajv.compile(pageSchema)

export function schemaForRelativePath(relativePath: string): object | null {
  const rel = relativePath.replace(/\\/g, '/')
  if (rel === 'site.json') return siteSchema as object
  if (rel.startsWith('pages/') && rel.endsWith('.json')) return pageSchema as object
  return null
}

export function lintJsonDocument(
  relativePath: string,
  text: string,
): { from: number; to: number; message: string; severity: 'error' | 'warning' }[] {
  const issues: { from: number; to: number; message: string; severity: 'error' | 'warning' }[] =
    []

  let parsed: unknown
  try {
    parsed = JSON.parse(text)
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    issues.push({ from: 0, to: Math.min(text.length, 1), message: msg, severity: 'error' })
    return issues
  }

  const schema = schemaForRelativePath(relativePath)
  if (!schema) return issues

  const validate = relativePath.replace(/\\/g, '/') === 'site.json' ? validateSiteDoc : validatePageDoc
  if (!validate(parsed)) {
    for (const e of validate.errors ?? []) {
      const path = e.instancePath || e.schemaPath
      const message = `${path}: ${e.message ?? 'invalid'}`
      issues.push({ from: 0, to: 1, message, severity: 'error' })
    }
  }

  return issues
}

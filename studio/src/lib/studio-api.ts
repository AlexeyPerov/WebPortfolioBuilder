import { invoke } from '@tauri-apps/api/core'

export type Severity = 'warning' | 'error'

export type Diagnostic = {
  file_path: string
  message: string
  severity: Severity
}

export type ProjectRootInfo = {
  project_root: string
  template_dir: string
}

export type ValidateSiteResult = {
  ok: boolean
  warnings: Diagnostic[]
  errors: Diagnostic[]
}

export type BuildSiteResult = {
  ok: boolean
  output_dir: string | null
  warnings: Diagnostic[]
  errors: Diagnostic[]
}

export type PreviewServerInfo = {
  url: string
  port: number
  output_dir: string
}

export function resolveProjectRoot() {
  return invoke<ProjectRootInfo>('resolve_project_root')
}

export function listContentBundles(projectRoot: string) {
  return invoke<string[]>('list_content_bundles', { projectRoot })
}

export function validateSite(projectRoot: string, sitePath: string, strict: boolean) {
  return invoke<ValidateSiteResult>('validate_site', { projectRoot, sitePath, strict })
}

export function buildSite(projectRoot: string, sitePath: string, strict: boolean) {
  return invoke<BuildSiteResult>('build_site', { projectRoot, sitePath, strict })
}

export function startPreviewServer(outputDir: string, port: number) {
  return invoke<PreviewServerInfo>('start_preview_server', { outputDir, port })
}

export function stopPreviewServer() {
  return invoke<void>('stop_preview_server')
}

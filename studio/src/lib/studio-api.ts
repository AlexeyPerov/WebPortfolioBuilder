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

export type BuiltinThemeId = 'dark-amber' | 'light-blue'

export type WorkspaceLayout = {
  sidebar_px?: number | null
  preview_px?: number | null
}

export type StudioSettings = {
  last_project_root?: string | null
  theme?: BuiltinThemeId | null
  workspace_layout?: WorkspaceLayout | null
}

export type WatchRebuildComplete = {
  build: BuildSiteResult
  preview: PreviewServerInfo | null
}

/** Matches backend `WATCH_DEBOUNCE_MS` — see studio/README.md */
export const AUTO_REBUILD_DEBOUNCE_MS = 500

export type BundleFileEntry = {
  relative_path: string
  name: string
  is_dir: boolean
}

export type BundleImagePreview = {
  relative_path: string
  data_url: string
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

export function getStudioSettings() {
  return invoke<StudioSettings>('get_studio_settings')
}

export function saveStudioSettings(settings: StudioSettings) {
  return invoke<void>('save_studio_settings', { settings })
}

export function projectInfoForRoot(projectRoot: string) {
  return invoke<ProjectRootInfo>('project_info_for_root', { projectRoot })
}

export function listBundleFiles(projectRoot: string, sitePath: string) {
  return invoke<BundleFileEntry[]>('list_bundle_files_cmd', { projectRoot, sitePath })
}

export function readBundleFile(
  projectRoot: string,
  sitePath: string,
  relativePath: string,
) {
  return invoke<string>('read_bundle_file_cmd', { projectRoot, sitePath, relativePath })
}

export function readBundleImage(
  projectRoot: string,
  sitePath: string,
  relativePath: string,
) {
  return invoke<BundleImagePreview>('read_bundle_image_cmd', {
    projectRoot,
    sitePath,
    relativePath,
  })
}

export function writeBundleFile(
  projectRoot: string,
  sitePath: string,
  relativePath: string,
  content: string,
) {
  return invoke<void>('write_bundle_file_cmd', {
    projectRoot,
    sitePath,
    relativePath,
    content,
  })
}

export function createSiteFromTemplate(projectRoot: string, siteId: string) {
  return invoke<string>('create_site_from_template', { projectRoot, siteId })
}

export function setAutoRebuild(
  enabled: boolean,
  projectRoot: string,
  sitePath: string,
  strict: boolean,
  previewPort: number,
) {
  return invoke<void>('set_auto_rebuild', {
    enabled,
    projectRoot,
    sitePath,
    strict,
    previewPort,
  })
}

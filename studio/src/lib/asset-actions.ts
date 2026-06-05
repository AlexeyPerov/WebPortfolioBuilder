import { confirm } from '@tauri-apps/plugin-dialog'
import { deleteBundleAsset } from './studio-api'

export async function copyAssetPathToClipboard(relativePath: string): Promise<void> {
  const path = relativePath.replace(/\\/g, '/')
  await navigator.clipboard.writeText(path)
}

export async function removeAssetWithConfirm(
  projectRoot: string,
  sitePath: string,
  relativePath: string,
): Promise<boolean> {
  const ok = await confirm(
    `Remove "${relativePath}" from assets? This deletes the file from disk.`,
    { title: 'Remove asset', kind: 'warning' },
  )
  if (!ok) return false

  await deleteBundleAsset(projectRoot, sitePath, relativePath)
  return true
}

export const IMAGE_DIALOG_FILTERS = [
  {
    name: 'Images',
    extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'ico', 'avif'],
  },
]

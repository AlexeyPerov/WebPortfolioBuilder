import { getContext, setContext } from 'svelte'

export type StudioBundleContext = {
  projectRoot: string
  sitePath: string
  imageAssets: string[]
  refreshFileTree: () => Promise<void>
}

const STUDIO_BUNDLE_CONTEXT_KEY = Symbol('studio-bundle')

export function setStudioBundleContext(ctx: StudioBundleContext) {
  setContext(STUDIO_BUNDLE_CONTEXT_KEY, ctx)
}

export function getStudioBundleContext(): StudioBundleContext | null {
  return getContext<StudioBundleContext | null>(STUDIO_BUNDLE_CONTEXT_KEY) ?? null
}

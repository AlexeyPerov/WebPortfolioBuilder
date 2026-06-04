<script lang="ts">
  import type { Snippet } from 'svelte'
  import {
    setStudioBundleContext,
    type StudioBundleContext,
  } from '../lib/studio-bundle-context'

  type Props = {
    projectRoot: string
    sitePath: string
    imageAssets: string[]
    refreshFileTree: () => Promise<void>
    children: Snippet
  }

  let { projectRoot, sitePath, imageAssets, refreshFileTree, children }: Props = $props()

  const ctx = $state<StudioBundleContext>({
    projectRoot: '',
    sitePath: '',
    imageAssets: [],
    refreshFileTree: async () => {},
  })

  setStudioBundleContext(ctx)

  $effect(() => {
    ctx.projectRoot = projectRoot
    ctx.sitePath = sitePath
    ctx.imageAssets = imageAssets
    ctx.refreshFileTree = refreshFileTree
  })
</script>

{@render children()}

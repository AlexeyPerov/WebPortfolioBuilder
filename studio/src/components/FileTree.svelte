<script lang="ts">
  import type { BundleFileEntry } from '../lib/studio-api'
  import { isImageFile, isJsonFile } from '../lib/image-files'

  type Props = {
    entries: BundleFileEntry[]
    selectedPath: string | null
    onselect?: (relativePath: string) => void
    onimportasset?: () => void
    onremoveasset?: (relativePath: string) => void
  }

  let { entries, selectedPath, onselect, onimportasset, onremoveasset }: Props = $props()

  let contextMenu = $state<{ path: string; x: number; y: number } | null>(null)

  function isSelectable(entry: BundleFileEntry) {
    return !entry.is_dir && (isJsonFile(entry.relative_path) || isImageFile(entry.relative_path))
  }

  function isAssetsDir(entry: BundleFileEntry) {
    return entry.is_dir && entry.relative_path === 'assets'
  }

  function onImageContextMenu(event: MouseEvent, relativePath: string) {
    event.preventDefault()
    contextMenu = { path: relativePath, x: event.clientX, y: event.clientY }
  }

  function closeContextMenu() {
    contextMenu = null
  }

  function removeFromMenu() {
    if (!contextMenu) return
    onremoveasset?.(contextMenu.path)
    closeContextMenu()
  }

  function onDocClick() {
    closeContextMenu()
  }

  function onDocKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') closeContextMenu()
  }
</script>

<svelte:window onclick={onDocClick} onkeydown={onDocKeydown} />

<nav class="file-tree" aria-label="Content bundle files">
  <ul>
    {#each entries as entry (entry.relative_path)}
      <li class:dir={entry.is_dir} class:nested={entry.relative_path.includes('/')}>
        {#if isSelectable(entry)}
          <button
            type="button"
            class:selected={selectedPath === entry.relative_path}
            class:image={isImageFile(entry.relative_path)}
            onclick={() => onselect?.(entry.relative_path)}
            oncontextmenu={(e) =>
              isImageFile(entry.relative_path) && onImageContextMenu(e, entry.relative_path)}
          >
            {entry.name}
          </button>
        {:else if isAssetsDir(entry)}
          <div class="dir-row">
            <span class="label">{entry.name}</span>
            {#if onimportasset}
              <button
                type="button"
                class="add-btn"
                title="Import image to assets"
                onclick={(e) => {
                  e.stopPropagation()
                  onimportasset()
                }}
              >
                +
              </button>
            {/if}
          </div>
        {:else}
          <span class="label">{entry.name}</span>
        {/if}
      </li>
    {/each}
  </ul>
</nav>

{#if contextMenu}
  <div
    class="context-menu"
    style:left="{contextMenu.x}px"
    style:top="{contextMenu.y}px"
    role="menu"
  >
    <button type="button" class="danger" role="menuitem" onclick={removeFromMenu}>Remove</button>
  </div>
{/if}

<style>
  .file-tree {
    overflow: auto;
    height: 100%;
    padding: 0.35rem 0;
    font-size: 0.85rem;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    margin: 0;
  }

  li.nested:not(.dir) {
    padding-left: 0.85rem;
  }

  li.dir + li.nested {
    padding-left: 0.85rem;
  }

  button,
  .label {
    display: block;
    width: 100%;
    text-align: left;
    padding: 0.2rem 0.65rem;
    border: none;
    background: transparent;
    font: inherit;
    color: inherit;
    cursor: default;
  }

  button {
    cursor: pointer;
    border-radius: 0.25rem;
  }

  button.image {
    font-family: ui-monospace, Consolas, monospace;
    font-size: 0.82rem;
  }

  button:hover {
    background: var(--color-hover);
  }

  button.selected {
    background: color-mix(in srgb, var(--color-accent) 22%, transparent);
    font-weight: 600;
  }

  .label {
    color: var(--color-text-secondary);
    font-weight: 600;
  }

  .dir-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-right: 0.35rem;
  }

  .dir-row .label {
    flex: 1;
    padding: 0.2rem 0.65rem;
  }

  .add-btn {
    width: auto;
    flex-shrink: 0;
    padding: 0.05rem 0.4rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.25rem;
    background: var(--color-hover);
    font-size: 0.85rem;
    font-weight: 600;
    line-height: 1.2;
    cursor: pointer;
  }

  .add-btn:hover {
    background: var(--color-pressed);
  }

  .context-menu {
    position: fixed;
    z-index: 500;
    min-width: 7rem;
    padding: 0.25rem 0;
    background: var(--color-surface-1);
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  }

  .context-menu button {
    width: 100%;
    padding: 0.35rem 0.75rem;
    border: none;
    background: transparent;
    text-align: left;
    font-size: 0.82rem;
    cursor: pointer;
  }

  .context-menu button:hover {
    background: var(--color-hover);
  }

  .context-menu button.danger {
    color: #e06c75;
  }
</style>

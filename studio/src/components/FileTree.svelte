<script lang="ts">
  import type { BundleFileEntry } from '../lib/studio-api'

  type Props = {
    entries: BundleFileEntry[]
    selectedPath: string | null
    onselect?: (relativePath: string) => void
  }

  let { entries, selectedPath, onselect }: Props = $props()

  function isSelectable(entry: BundleFileEntry) {
    return !entry.is_dir && entry.relative_path.endsWith('.json')
  }
</script>

<nav class="file-tree" aria-label="Content bundle files">
  <ul>
    {#each entries as entry (entry.relative_path)}
      <li class:dir={entry.is_dir} class:nested={entry.relative_path.includes('/')}>
        {#if isSelectable(entry)}
          <button
            type="button"
            class:selected={selectedPath === entry.relative_path}
            onclick={() => onselect?.(entry.relative_path)}
          >
            {entry.name}
          </button>
        {:else}
          <span class="label">{entry.name}</span>
        {/if}
      </li>
    {/each}
  </ul>
</nav>

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

  button:hover {
    background: #e8ecf2;
  }

  button.selected {
    background: #d6e4f5;
    font-weight: 600;
  }

  .label {
    color: #5c6570;
    font-weight: 600;
  }
</style>

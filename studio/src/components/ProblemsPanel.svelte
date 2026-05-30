<script lang="ts">
  import type { Diagnostic } from '../lib/studio-api'

  type Props = {
    items: Diagnostic[]
  }

  let { items }: Props = $props()
</script>

<section class="problems" aria-label="Problems">
  <header>
    <h2>Problems</h2>
    <span class="count">{items.length}</span>
  </header>
  {#if items.length === 0}
    <p class="empty">No problems.</p>
  {:else}
    <ul>
      {#each items as item, i (`${item.file_path}-${item.message}-${i}`)}
        <li class={item.severity}>
          {#if item.file_path}
            <span class="file">{item.file_path}</span>
          {/if}
          <span class="msg">{item.message}</span>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .problems {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-top: 1px solid var(--color-border-subtle);
    background: var(--color-statusbar-bg);
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 0.65rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  h2 {
    margin: 0;
    font-size: 0.72rem;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-secondary);
  }

  .count {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }

  .empty {
    margin: 0;
    padding: 0.5rem 0.65rem;
    font-size: 0.8rem;
    color: var(--color-text-secondary);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow: auto;
    max-height: 9rem;
  }

  li {
    padding: 0.25rem 0.65rem;
    font-size: 0.78rem;
    border-bottom: 1px solid var(--color-border-subtle);
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  li.warning {
    background: color-mix(in srgb, #ffc800 18%, var(--color-surface-1));
  }

  li.error {
    background: color-mix(in srgb, #e06c75 14%, var(--color-surface-1));
  }

  .file {
    font-family: ui-monospace, Consolas, monospace;
    color: var(--color-accent);
    flex: 0 0 auto;
  }

  .msg {
    flex: 1 1 auto;
  }
</style>

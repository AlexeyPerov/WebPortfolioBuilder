<script lang="ts">
  type Props = {
    lines: string[]
    collapsed?: boolean
    onTogglePanel?: () => void
  }

  let { lines, collapsed = false, onTogglePanel }: Props = $props()

  const lastLine = $derived(lines.length > 0 ? lines[lines.length - 1] : 'No log entries yet.')
</script>

<section class="build-log" aria-label="Logs">
  {#if collapsed}
    <button
      type="button"
      class="log-bar"
      title="Expand logs"
      aria-expanded="false"
      onclick={() => onTogglePanel?.()}
    >
      <span class="logs-label">Logs</span>
      <span class="last-line" title={lastLine}>{lastLine}</span>
    </button>
  {:else}
    <button
      type="button"
      class="log-header"
      title="Collapse logs"
      aria-expanded="true"
      onclick={() => onTogglePanel?.()}
    >
      <h2>Logs</h2>
      <span class="chevron" aria-hidden="true">▾</span>
    </button>
    <pre>{lines.length > 0 ? lines.join('\n') : 'No log entries yet.'}</pre>
  {/if}
</section>

<style>
  .build-log {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
    background: var(--color-surface-1);
  }

  .log-header,
  .log-bar {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    width: 100%;
    margin: 0;
    padding: 0;
    border: none;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-surface-1);
    cursor: pointer;
    text-align: left;
    font: inherit;
    color: inherit;
  }

  .log-bar {
    min-width: 0;
    border-bottom: none;
    padding: 0 0.65rem;
    line-height: 32px;
  }

  .log-header:hover,
  .log-bar:hover {
    background: var(--color-hover);
  }

  h2 {
    margin: 0;
    padding: 0.35rem 0.65rem;
    font-size: 0.72rem;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-secondary);
    pointer-events: none;
  }

  .chevron {
    margin-left: auto;
    margin-right: 0.45rem;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    pointer-events: none;
  }

  .last-line {
    flex: 1;
    min-width: 0;
    font-size: 0.72rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--color-text-secondary);
    font-family: ui-monospace, Consolas, monospace;
    pointer-events: none;
  }

  .logs-label {
    flex-shrink: 0;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-right: 0.35rem;
    color: var(--color-text-secondary);
    font-family: inherit;
    pointer-events: none;
  }

  pre {
    margin: 0;
    flex: 1;
    overflow: auto;
    padding: 0.45rem 0.65rem;
    font-size: 0.75rem;
    line-height: 1.35;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>

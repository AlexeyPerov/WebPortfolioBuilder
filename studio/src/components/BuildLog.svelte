<script lang="ts">
  type Props = {
    lines: string[]
  }

  let { lines }: Props = $props()
</script>

<section class="build-log" aria-label="Build log">
  <h2>Build log</h2>
  <pre>{lines.length > 0 ? lines.join('\n') : 'No log entries yet.'}</pre>
</section>

<style>
  .build-log {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-top: 1px solid var(--color-border-subtle);
    background: var(--color-surface-1);
  }

  h2 {
    margin: 0;
    padding: 0.35rem 0.65rem;
    font-size: 0.72rem;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-secondary);
    border-bottom: 1px solid var(--color-border-subtle);
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

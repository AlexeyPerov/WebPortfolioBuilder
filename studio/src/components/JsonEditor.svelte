<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { EditorState, type Extension } from '@codemirror/state'
  import { EditorView, keymap } from '@codemirror/view'
  import { json } from '@codemirror/lang-json'
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
  import { linter, type Diagnostic as CmDiagnostic } from '@codemirror/lint'
  import { lintJsonDocument } from '../lib/editor-schema'

  type Props = {
    relativePath: string
    value: string
    onchange?: (value: string) => void
  }

  let { relativePath, value, onchange }: Props = $props()

  let host = $state<HTMLDivElement | null>(null)
  let view: EditorView | null = null

  function jsonLinter() {
    return linter((view) => {
      const text = view.state.doc.toString()
      return lintJsonDocument(relativePath, text).map(
        (issue): CmDiagnostic => ({
          from: issue.from,
          to: issue.to,
          severity: issue.severity,
          message: issue.message,
        }),
      )
    })
  }

  function buildExtensions(): Extension[] {
    return [
      history(),
      json(),
      jsonLinter(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          onchange?.(update.state.doc.toString())
        }
      }),
      EditorView.theme({
        '&': { height: '100%', fontSize: '13px' },
        '.cm-scroller': { fontFamily: 'ui-monospace, Consolas, monospace' },
      }),
    ]
  }

  onMount(() => {
    if (!host) return
    const state = EditorState.create({
      doc: value,
      extensions: buildExtensions(),
    })
    view = new EditorView({ state, parent: host })
  })

  onDestroy(() => {
    view?.destroy()
    view = null
  })

  $effect(() => {
    if (!view) return
    const current = view.state.doc.toString()
    if (current !== value) {
      view.dispatch({
        changes: { from: 0, to: current.length, insert: value },
      })
    }
  })

  $effect(() => {
    if (!view) return
    // Re-run linter when path changes (schema selection).
    view.requestMeasure()
  })
</script>

<div class="editor-host" bind:this={host}></div>

<style>
  .editor-host {
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .editor-host :global(.cm-editor) {
    height: 100%;
  }

  .editor-host :global(.cm-focused) {
    outline: none;
  }
</style>

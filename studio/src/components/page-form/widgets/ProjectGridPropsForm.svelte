<script lang="ts">
  import { mergeWidgetProps } from '../../../lib/page-form'
  import {
    readProjectGridCards,
    readString,
    writeProjectGridCard,
    type ProjectGridCard,
  } from '../../../lib/widget-props'
  import AssetPathField from '../fields/AssetPathField.svelte'
  import StringListField from '../fields/StringListField.svelte'

  type Props = {
    props: Record<string, unknown>
    onchange?: (props: Record<string, unknown>) => void
  }

  let { props, onchange }: Props = $props()

  const cards = $derived(readProjectGridCards(props))

  function emit(patch: Record<string, unknown>) {
    onchange?.(mergeWidgetProps(props, patch))
  }

  function updateCards(next: ProjectGridCard[]) {
    emit({ cards: next.map(writeProjectGridCard) })
  }

  function updateCard(index: number, patch: Partial<ProjectGridCard>) {
    updateCards(cards.map((card, i) => (i === index ? { ...card, ...patch } : card)))
  }

  function addMetaPair(index: number) {
    const card = cards[index]
    if (!card) return
    updateCard(index, { metaPairs: [...card.metaPairs, { key: '', value: '' }] })
  }

  function updateMetaPair(cardIndex: number, pairIndex: number, patch: Partial<{ key: string; value: string }>) {
    const card = cards[cardIndex]
    if (!card) return
    updateCard(cardIndex, {
      metaPairs: card.metaPairs.map((pair, i) =>
        i === pairIndex ? { ...pair, ...patch } : pair,
      ),
    })
  }

  function removeMetaPair(cardIndex: number, pairIndex: number) {
    const card = cards[cardIndex]
    if (!card) return
    updateCard(cardIndex, { metaPairs: card.metaPairs.filter((_, i) => i !== pairIndex) })
  }

  function addCard() {
    updateCards([
      ...cards,
      {
        title: '',
        description: '',
        tags: [],
        image: '',
        metaMode: 'string',
        metaString: '',
        metaPairs: [],
        cta: { label: '', url: '' },
      },
    ])
  }

  function removeCard(index: number) {
    updateCards(cards.filter((_, i) => i !== index))
  }
</script>

<div class="widget-props-form">
  <label class="field"><span class="label">Heading</span><input type="text" value={readString(props, 'heading')} oninput={(e) => emit({ heading: e.currentTarget.value })} /></label>
  <label class="field"><span class="label">Subheading</span><input type="text" value={readString(props, 'subheading')} oninput={(e) => emit({ subheading: e.currentTarget.value })} /></label>
  <label class="field"><span class="label">Section ID</span><input type="text" spellcheck="false" value={readString(props, 'section_id')} oninput={(e) => emit({ section_id: e.currentTarget.value })} /></label>
  <label class="field"><span class="label">Min card column width</span><input type="text" spellcheck="false" value={readString(props, 'min_card_column_width')} oninput={(e) => emit({ min_card_column_width: e.currentTarget.value })} /></label>

  {#each cards as card, index (index)}
    <details class="card-panel" open={index === 0}>
      <summary>Card {index + 1}{card.title ? `: ${card.title}` : ''}</summary>
      <div class="card-body">
        <label class="field"><span class="label">Title</span><input type="text" value={card.title} oninput={(e) => updateCard(index, { title: e.currentTarget.value })} /></label>
        <label class="field"><span class="label">Description</span><textarea rows="2" value={card.description} oninput={(e) => updateCard(index, { description: e.currentTarget.value })}></textarea></label>
        <StringListField label="Tags" value={card.tags} onchange={(tags) => updateCard(index, { tags })} />
        <AssetPathField label="Image" value={card.image} onchange={(image) => updateCard(index, { image })} />

        <fieldset class="meta-fieldset">
          <legend>Meta</legend>
          <div class="meta-mode">
            <label><input type="radio" checked={card.metaMode === 'string'} onchange={() => updateCard(index, { metaMode: 'string' })} /> Text line</label>
            <label><input type="radio" checked={card.metaMode === 'object'} onchange={() => updateCard(index, { metaMode: 'object' })} /> Key-value pairs</label>
          </div>
          {#if card.metaMode === 'string'}
            <input type="text" value={card.metaString} oninput={(e) => updateCard(index, { metaString: e.currentTarget.value })} />
          {:else}
            {#each card.metaPairs as pair, pairIndex (pairIndex)}
              <div class="meta-pair">
                <input type="text" placeholder="Key" value={pair.key} oninput={(e) => updateMetaPair(index, pairIndex, { key: e.currentTarget.value })} />
                <input type="text" placeholder="Value" value={pair.value} oninput={(e) => updateMetaPair(index, pairIndex, { value: e.currentTarget.value })} />
                <button type="button" class="icon-btn danger" onclick={() => removeMetaPair(index, pairIndex)}>×</button>
              </div>
            {/each}
            <button type="button" class="text-btn" onclick={() => addMetaPair(index)}>Add meta field</button>
          {/if}
        </fieldset>

        <fieldset class="cta-fieldset">
          <legend>Call to action</legend>
          <label class="field"><span class="label">Label</span><input type="text" value={card.cta.label} oninput={(e) => updateCard(index, { cta: { ...card.cta, label: e.currentTarget.value } })} /></label>
          <label class="field"><span class="label">URL</span><input type="text" spellcheck="false" value={card.cta.url} oninput={(e) => updateCard(index, { cta: { ...card.cta, url: e.currentTarget.value } })} /></label>
        </fieldset>

        <button type="button" class="danger-btn" onclick={() => removeCard(index)}>Remove card</button>
      </div>
    </details>
  {/each}

  <button type="button" class="secondary" onclick={addCard}>Add card</button>
</div>

<style>
  .widget-props-form {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    margin-top: 0.65rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .label {
    font-weight: 500;
    font-size: 0.82rem;
  }

  input[type='text'],
  textarea {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
  }

  .card-panel {
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    padding: 0.35rem 0.55rem;
    background: var(--color-surface-1);
  }

  .card-panel summary {
    cursor: pointer;
    font-weight: 600;
    font-size: 0.82rem;
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.5rem 0 0.25rem;
  }

  .meta-fieldset,
  .cta-fieldset {
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    padding: 0.45rem 0.55rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .meta-fieldset legend,
  .cta-fieldset legend {
    font-size: 0.78rem;
    font-weight: 600;
    padding: 0 0.25rem;
  }

  .meta-mode {
    display: flex;
    gap: 1rem;
    font-size: 0.82rem;
  }

  .meta-pair {
    display: flex;
    gap: 0.35rem;
    align-items: center;
  }

  .meta-pair input {
    flex: 1;
    min-width: 0;
  }

  .text-btn {
    align-self: flex-start;
    padding: 0;
    border: none;
    background: none;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    text-decoration: underline;
    cursor: pointer;
  }

  .icon-btn {
    padding: 0.15rem 0.4rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.25rem;
    background: var(--color-surface-1);
    cursor: pointer;
  }

  .icon-btn.danger {
    color: #e06c75;
  }

  .secondary,
  .danger-btn {
    align-self: flex-start;
    padding: 0.35rem 0.65rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    font-size: 0.82rem;
    cursor: pointer;
  }

  .secondary {
    background: var(--color-hover);
  }

  .danger-btn {
    background: color-mix(in srgb, #e06c75 12%, var(--color-surface-1));
    color: #e06c75;
  }
</style>

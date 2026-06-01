<script lang="ts">
  import ColorFieldInput from '../../ColorFieldInput.svelte'
  import { defaultPropsForType, mergeWidgetProps } from '../../../lib/page-form'
  import { showThemeColorPicker } from '../../../lib/color-value'
  import {
    readCatalogApps,
    readString,
    writeCatalogApp,
    type CatalogApp,
  } from '../../../lib/widget-props'
  import AssetPathField from '../fields/AssetPathField.svelte'
  import StringListField from '../fields/StringListField.svelte'

  type Props = {
    props: Record<string, unknown>
    onchange?: (props: Record<string, unknown>) => void
  }

  let { props, onchange }: Props = $props()

  const apps = $derived(readCatalogApps(props))

  function emit(patch: Record<string, unknown>) {
    onchange?.(mergeWidgetProps(props, patch))
  }

  function updateApps(next: CatalogApp[]) {
    emit({ apps: next.map(writeCatalogApp) })
  }

  function updateApp(index: number, patch: Partial<CatalogApp>) {
    updateApps(apps.map((app, i) => (i === index ? { ...app, ...patch } : app)))
  }

  function addApp() {
    const defaults = readCatalogApps(defaultPropsForType('apps_showcase'))
    const blank = defaults[0] ?? {
      image: '',
      header_image: '',
      swiper_images: [],
      card_background: '',
      title: '',
      text_1: '',
      text_2: '',
      stat_left_line_1: '',
      stat_left_line_2: '',
      stat_right_line_1: '',
      stat_right_line_2: '',
      google_play_url: '',
      app_store_url: '',
      amazon_store_url: '',
      galaxy_store_url: '',
    }
    updateApps([...apps, blank])
  }

  function removeApp(index: number) {
    updateApps(apps.filter((_, i) => i !== index))
  }
</script>

<div class="widget-props-form">
  <label class="field">
    <span class="label">Section title</span>
    <input
      type="text"
      value={readString(props, 'section_title')}
      oninput={(e) => emit({ section_title: e.currentTarget.value })}
    />
  </label>

  {#each apps as app, index (index)}
    <details class="app-card" open={index === 0}>
      <summary>App {index + 1}{app.title ? `: ${app.title}` : ''}</summary>
      <div class="card-body">
        <AssetPathField label="Icon" value={app.image} onchange={(v) => updateApp(index, { image: v })} />
        <AssetPathField label="Header image" value={app.header_image} onchange={(v) => updateApp(index, { header_image: v })} />
        <StringListField
          label="Swiper images"
          addLabel="Add slide"
          value={app.swiper_images}
          onchange={(swiper_images) => updateApp(index, { swiper_images })}
        />
        <label class="field">
          <span class="label">Card background</span>
          {#if showThemeColorPicker('card_background', app.card_background)}
            <ColorFieldInput
              tokenKey="card_background"
              value={app.card_background}
              onchange={(v) => updateApp(index, { card_background: v })}
            />
          {:else}
            <input type="text" value={app.card_background} spellcheck="false" oninput={(e) => updateApp(index, { card_background: e.currentTarget.value })} />
          {/if}
        </label>
        <label class="field"><span class="label">Title</span><input type="text" value={app.title} oninput={(e) => updateApp(index, { title: e.currentTarget.value })} /></label>
        <label class="field"><span class="label">Text 1</span><textarea rows="2" value={app.text_1} oninput={(e) => updateApp(index, { text_1: e.currentTarget.value })}></textarea></label>
        <label class="field"><span class="label">Text 2</span><textarea rows="2" value={app.text_2} oninput={(e) => updateApp(index, { text_2: e.currentTarget.value })}></textarea></label>
        <div class="stat-grid">
          <label class="field"><span class="label">Stat left 1</span><input type="text" value={app.stat_left_line_1} oninput={(e) => updateApp(index, { stat_left_line_1: e.currentTarget.value })} /></label>
          <label class="field"><span class="label">Stat left 2</span><input type="text" value={app.stat_left_line_2} oninput={(e) => updateApp(index, { stat_left_line_2: e.currentTarget.value })} /></label>
          <label class="field"><span class="label">Stat right 1</span><input type="text" value={app.stat_right_line_1} oninput={(e) => updateApp(index, { stat_right_line_1: e.currentTarget.value })} /></label>
          <label class="field"><span class="label">Stat right 2</span><input type="text" value={app.stat_right_line_2} oninput={(e) => updateApp(index, { stat_right_line_2: e.currentTarget.value })} /></label>
        </div>
        <label class="field"><span class="label">Google Play URL</span><input type="text" spellcheck="false" value={app.google_play_url} oninput={(e) => updateApp(index, { google_play_url: e.currentTarget.value })} /></label>
        <label class="field"><span class="label">App Store URL</span><input type="text" spellcheck="false" value={app.app_store_url} oninput={(e) => updateApp(index, { app_store_url: e.currentTarget.value })} /></label>
        <label class="field"><span class="label">Amazon Store URL</span><input type="text" spellcheck="false" value={app.amazon_store_url} oninput={(e) => updateApp(index, { amazon_store_url: e.currentTarget.value })} /></label>
        <label class="field"><span class="label">Galaxy Store URL</span><input type="text" spellcheck="false" value={app.galaxy_store_url} oninput={(e) => updateApp(index, { galaxy_store_url: e.currentTarget.value })} /></label>
        <button type="button" class="danger-btn" onclick={() => removeApp(index)}>Remove app</button>
      </div>
    </details>
  {/each}

  <button type="button" class="secondary" onclick={addApp}>Add app</button>
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

  .app-card {
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    padding: 0.35rem 0.55rem;
    background: var(--color-surface-1);
  }

  .app-card summary {
    cursor: pointer;
    font-weight: 600;
    font-size: 0.82rem;
    padding: 0.25rem 0;
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.5rem 0 0.25rem;
  }

  .stat-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    gap: 0.45rem;
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

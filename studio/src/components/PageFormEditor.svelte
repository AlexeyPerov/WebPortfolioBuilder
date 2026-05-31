<script lang="ts">
  import {
    applyPageForm,
    parsePageForm,
    type PageFormModel,
    type PageLayout,
    type PageSeo,
  } from '../lib/page-form'

  type Props = {
    value: string
    onchange?: (value: string) => void
  }

  let { value, onchange }: Props = $props()

  const parsed = $derived(parsePageForm(value))

  function emitModel(model: PageFormModel) {
    if (!parsed.ok) return
    onchange?.(applyPageForm(parsed.doc, model))
  }

  function updateField<K extends keyof PageFormModel>(key: K, nextValue: PageFormModel[K]) {
    if (!parsed.ok) return
    emitModel({ ...parsed.model, [key]: nextValue })
  }

  function updateSeo(key: keyof PageSeo, nextValue: string) {
    if (!parsed.ok) return
    emitModel({
      ...parsed.model,
      seo: { ...parsed.model.seo, [key]: nextValue },
    })
  }

  function updateLayout(key: keyof PageLayout, nextValue: boolean) {
    if (!parsed.ok) return
    emitModel({
      ...parsed.model,
      layout: { ...parsed.model.layout, [key]: nextValue },
    })
  }
</script>

<div class="page-form">
  {#if !parsed.ok}
    <div class="form-error" role="alert">
      <strong>Form unavailable</strong>
      <p>Fix JSON syntax errors in the JSON tab before using the form editor.</p>
      <p class="detail">{parsed.error}</p>
    </div>
  {:else}
    <section class="form-section">
      <div class="section-head">
        <h2>Page</h2>
        <p class="hint">URL slug and document title for this page file.</p>
      </div>

      <div class="field-grid">
        <label class="field">
          <span class="label">Slug</span>
          <span class="hint-inline">Empty string maps to <code>index.html</code> at the site root.</span>
          <input
            type="text"
            value={parsed.model.slug}
            spellcheck="false"
            oninput={(e) => updateField('slug', e.currentTarget.value)}
          />
        </label>

        <label class="field">
          <span class="label">Title</span>
          <span class="hint-inline">Document title and default Open Graph title.</span>
          <input
            type="text"
            value={parsed.model.title}
            oninput={(e) => updateField('title', e.currentTarget.value)}
          />
        </label>
      </div>
    </section>

    <section class="form-section">
      <div class="section-head">
        <h2>SEO</h2>
        <p class="hint">Optional metadata under <code>seo</code>.</p>
      </div>

      <div class="field-grid">
        <label class="field span-all">
          <span class="label">Description</span>
          <textarea
            rows="3"
            value={parsed.model.seo.description}
            oninput={(e) => updateSeo('description', e.currentTarget.value)}
          ></textarea>
        </label>

        <label class="field">
          <span class="label">Open Graph image</span>
          <span class="hint-inline">Asset path, e.g. <code>assets/logo.png</code></span>
          <input
            type="text"
            value={parsed.model.seo.og_image}
            spellcheck="false"
            oninput={(e) => updateSeo('og_image', e.currentTarget.value)}
          />
        </label>

        <label class="field">
          <span class="label">Canonical URL</span>
          <input
            type="text"
            value={parsed.model.seo.canonical_url}
            spellcheck="false"
            oninput={(e) => updateSeo('canonical_url', e.currentTarget.value)}
          />
        </label>
      </div>
    </section>

    <section class="form-section">
      <div class="section-head">
        <h2>Layout</h2>
        <p class="hint">Hide site chrome on this page.</p>
      </div>

      <div class="checkbox-row">
        <label class="checkbox-field">
          <input
            type="checkbox"
            checked={parsed.model.layout.hide_header}
            onchange={(e) => updateLayout('hide_header', e.currentTarget.checked)}
          />
          Hide header
        </label>

        <label class="checkbox-field">
          <input
            type="checkbox"
            checked={parsed.model.layout.hide_footer}
            onchange={(e) => updateLayout('hide_footer', e.currentTarget.checked)}
          />
          Hide footer
        </label>
      </div>
    </section>

    <p class="json-only-note">
      The <code>widgets</code> array remains editable in the JSON tab only.
    </p>
  {/if}
</div>

<style>
  .page-form {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 1rem 1.1rem 1.5rem;
    font-size: 0.875rem;
  }

  .form-error {
    margin: 1rem 0;
    padding: 0.85rem 1rem;
    border: 1px solid color-mix(in srgb, #e06c75 45%, var(--color-border-subtle));
    border-radius: 0.4rem;
    background: color-mix(in srgb, #e06c75 10%, var(--color-surface-1));
    color: var(--color-text-primary);
  }

  .form-error p {
    margin: 0.35rem 0 0;
  }

  .detail {
    font-family: ui-monospace, Consolas, monospace;
    font-size: 0.8rem;
  }

  .form-section + .form-section {
    margin-top: 1.5rem;
    padding-top: 1.25rem;
    border-top: 1px solid var(--color-border-subtle);
  }

  .section-head h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .hint {
    margin: 0.25rem 0 0.85rem;
    color: var(--color-text-secondary);
    font-size: 0.8rem;
  }

  .field-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
    gap: 0.65rem 0.85rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .field.span-all {
    grid-column: 1 / -1;
  }

  .field .label {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .hint-inline {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }

  input[type='text'],
  textarea {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
    font-family: ui-monospace, Consolas, monospace;
  }

  textarea {
    resize: vertical;
    min-height: 4rem;
    font-family: inherit;
  }

  .checkbox-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem 1.5rem;
  }

  .checkbox-field {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .json-only-note {
    margin: 1.25rem 0 0;
    padding: 0.65rem 0.75rem;
    border-radius: 0.35rem;
    background: var(--color-statusbar-bg);
    color: var(--color-text-secondary);
    font-size: 0.78rem;
  }
</style>

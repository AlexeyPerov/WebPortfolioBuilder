<script lang="ts">
  import ColorFieldInput from './ColorFieldInput.svelte'
  import {
    applySiteForm,
    mergeThemeForEdit,
    newEmptyNavItem,
    parseSiteForm,
    themeTokenLabel,
    type NavItem,
    type SiteFormModel,
  } from '../lib/site-form'
  import { showThemeColorPicker } from '../lib/color-value'

  type Props = {
    value: string
    onchange?: (value: string) => void
  }

  let { value, onchange }: Props = $props()

  const parsed = $derived(parseSiteForm(value))

  function emitModel(model: SiteFormModel) {
    if (!parsed.ok) return
    onchange?.(applySiteForm(parsed.doc, model))
  }

  function updateTheme(key: string, nextValue: string) {
    if (!parsed.ok) return
    emitModel({
      ...parsed.model,
      theme: { ...parsed.model.theme, [key]: nextValue },
    })
  }

  function removeThemeKey(key: string) {
    if (!parsed.ok) return
    const theme = { ...parsed.model.theme }
    delete theme[key]
    emitModel({ ...parsed.model, theme })
  }

  function addThemeToken() {
    if (!parsed.ok) return
    let key = 'custom_token'
    let n = 1
    while (key in parsed.model.theme) {
      key = `custom_token_${n++}`
    }
    emitModel({
      ...parsed.model,
      theme: { ...parsed.model.theme, [key]: '' },
    })
  }

  function updateNav(index: number, patch: Partial<NavItem>) {
    if (!parsed.ok) return
    const nav = parsed.model.nav.map((item, i) =>
      i === index ? { ...item, ...patch } : item,
    )
    emitModel({ ...parsed.model, nav })
  }

  function addNavRow() {
    if (!parsed.ok) return
    emitModel({ ...parsed.model, nav: [...parsed.model.nav, newEmptyNavItem()] })
  }

  function removeNavRow(index: number) {
    if (!parsed.ok) return
    emitModel({
      ...parsed.model,
      nav: parsed.model.nav.filter((_, i) => i !== index),
    })
  }

  function moveNavRow(index: number, delta: number) {
    if (!parsed.ok) return
    const nav = [...parsed.model.nav]
    const target = index + delta
    if (target < 0 || target >= nav.length) return
    ;[nav[index], nav[target]] = [nav[target], nav[index]]
    emitModel({ ...parsed.model, nav })
  }

  const themeRows = $derived(parsed.ok ? mergeThemeForEdit(parsed.model.theme) : null)
</script>

<div class="site-form">
  {#if !parsed.ok}
    <div class="form-error" role="alert">
      <strong>Form unavailable</strong>
      <p>Fix JSON syntax errors in the JSON tab before using the form editor.</p>
      <p class="detail">{parsed.error}</p>
    </div>
  {:else}
    <section class="form-section">
      <div class="section-head">
        <h2>Theme</h2>
        <p class="hint">CSS custom properties injected into <code>:root</code>.</p>
      </div>

      <div class="theme-grid">
        {#each themeRows!.known as row (row.key)}
          <label class="field">
            <span class="label">{themeTokenLabel(row.key)}</span>
            <span class="token-name">{row.key}</span>
            {#if showThemeColorPicker(row.key, row.value)}
              <ColorFieldInput
                tokenKey={row.key}
                value={row.value}
                onchange={(nextValue) => updateTheme(row.key, nextValue)}
              />
            {:else}
              <input
                type="text"
                value={row.value}
                spellcheck="false"
                oninput={(e) => updateTheme(row.key, e.currentTarget.value)}
              />
            {/if}
          </label>
        {/each}
      </div>

      {#if themeRows!.extra.length > 0}
        <h3 class="subhead">Additional tokens</h3>
        <div class="theme-grid">
          {#each themeRows!.extra as row (row.key)}
            <div class="field extra-token">
              <span class="label">{row.key}</span>
              {#if showThemeColorPicker(row.key, row.value)}
                <ColorFieldInput
                  tokenKey={row.key}
                  value={row.value}
                  onchange={(nextValue) => updateTheme(row.key, nextValue)}
                />
              {:else}
                <input
                  type="text"
                  value={row.value}
                  spellcheck="false"
                  oninput={(e) => updateTheme(row.key, e.currentTarget.value)}
                />
              {/if}
              <button type="button" class="icon-btn" title="Remove token" onclick={() => removeThemeKey(row.key)}>
                ×
              </button>
            </div>
          {/each}
        </div>
      {/if}

      <button type="button" class="secondary" onclick={addThemeToken}>Add theme token</button>
    </section>

    <section class="form-section">
      <div class="section-head">
        <h2>Header navigation</h2>
        <p class="hint">Items rendered in the site header (<code>header.nav</code>).</p>
      </div>

      {#if parsed.model.nav.length === 0}
        <p class="empty">No navigation items yet.</p>
      {:else}
        <div class="nav-table-wrap">
          <table class="nav-table">
            <thead>
              <tr>
                <th scope="col">Label</th>
                <th scope="col">Href</th>
                <th scope="col">New tab</th>
                <th scope="col"><span class="sr-only">Actions</span></th>
              </tr>
            </thead>
            <tbody>
              {#each parsed.model.nav as item, index (index)}
                <tr>
                  <td>
                    <input
                      type="text"
                      value={item.label}
                      oninput={(e) => updateNav(index, { label: e.currentTarget.value })}
                    />
                  </td>
                  <td>
                    <input
                      type="text"
                      value={item.href}
                      spellcheck="false"
                      oninput={(e) => updateNav(index, { href: e.currentTarget.value })}
                    />
                  </td>
                  <td class="center">
                    <input
                      type="checkbox"
                      checked={item.open_in_new_tab}
                      onchange={(e) =>
                        updateNav(index, { open_in_new_tab: e.currentTarget.checked })}
                    />
                  </td>
                  <td class="row-actions">
                    <button
                      type="button"
                      class="icon-btn"
                      title="Move up"
                      disabled={index === 0}
                      onclick={() => moveNavRow(index, -1)}>↑</button
                    >
                    <button
                      type="button"
                      class="icon-btn"
                      title="Move down"
                      disabled={index === parsed.model.nav.length - 1}
                      onclick={() => moveNavRow(index, 1)}>↓</button
                    >
                    <button
                      type="button"
                      class="icon-btn danger"
                      title="Remove row"
                      onclick={() => removeNavRow(index)}>×</button
                    >
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}

      <button type="button" class="secondary" onclick={addNavRow}>Add nav item</button>
    </section>

    <p class="json-only-note">
      Other <code>site.json</code> sections (footer, widgets, typography, …) remain editable in
      the JSON tab only.
    </p>
  {/if}
</div>

<style>
  .site-form {
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

  .subhead {
    margin: 1rem 0 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
    gap: 0.65rem 0.85rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .field .label {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .token-name {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
    font-family: ui-monospace, Consolas, monospace;
  }

  .extra-token {
    position: relative;
  }

  .extra-token .icon-btn {
    position: absolute;
    top: 0;
    right: 0;
  }

  input[type='text'] {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
    font-family: ui-monospace, Consolas, monospace;
  }

  .secondary {
    margin-top: 0.75rem;
    padding: 0.35rem 0.65rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    background: var(--color-hover);
    font-size: 0.82rem;
    cursor: pointer;
  }

  .secondary:hover {
    background: var(--color-pressed);
  }

  .nav-table-wrap {
    overflow-x: auto;
  }

  .nav-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }

  .nav-table th,
  .nav-table td {
    padding: 0.35rem 0.4rem;
    border-bottom: 1px solid var(--color-border-subtle);
    text-align: left;
    vertical-align: middle;
  }

  .nav-table th {
    font-weight: 600;
    color: var(--color-text-secondary);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }

  .nav-table input[type='text'] {
    min-width: 6rem;
  }

  .center {
    text-align: center;
  }

  .row-actions {
    white-space: nowrap;
    width: 1%;
  }

  .icon-btn {
    padding: 0.15rem 0.4rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.25rem;
    background: var(--color-surface-1);
    cursor: pointer;
    font-size: 0.85rem;
    line-height: 1.2;
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--color-hover);
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .icon-btn.danger {
    color: #e06c75;
  }

  .empty {
    margin: 0 0 0.5rem;
    color: var(--color-text-secondary);
    font-style: italic;
  }

  .json-only-note {
    margin: 1.25rem 0 0;
    padding: 0.65rem 0.75rem;
    border-radius: 0.35rem;
    background: var(--color-statusbar-bg);
    color: var(--color-text-secondary);
    font-size: 0.78rem;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>

<script lang="ts">
  import { defaultPropsForType, type WidgetNode } from '../../lib/page-form'
  import {
    isLayoutWidget,
    WIDGET_TYPE_IDS,
    widgetTypeLabel,
  } from '../../lib/widget-types'
  import WidgetPropsJsonFallback from './WidgetPropsJsonFallback.svelte'
  import AppsShowcasePropsForm from './widgets/AppsShowcasePropsForm.svelte'
  import CareersTabsPropsForm from './widgets/CareersTabsPropsForm.svelte'
  import CoverBannerPropsForm from './widgets/CoverBannerPropsForm.svelte'
  import FollowUsPropsForm from './widgets/FollowUsPropsForm.svelte'
  import ImagesGridPropsForm from './widgets/ImagesGridPropsForm.svelte'
  import InfoGridPropsForm from './widgets/InfoGridPropsForm.svelte'
  import IntroPropsForm from './widgets/IntroPropsForm.svelte'
  import MediaSwiperPropsForm from './widgets/MediaSwiperPropsForm.svelte'
  import ProjectGridPropsForm from './widgets/ProjectGridPropsForm.svelte'

  type Props = {
    widget: WidgetNode
    onchange?: (widget: WidgetNode) => void
  }

  let { widget, onchange }: Props = $props()

  function emit(patch: Partial<WidgetNode>) {
    onchange?.({ ...widget, ...patch })
  }

  function propsHaveContent(props: Record<string, unknown>): boolean {
    return Object.keys(props).length > 0
  }

  function changeType(nextType: string) {
    if (nextType === widget.type) return
    if (propsHaveContent(widget.props)) {
      const label = widgetTypeLabel(nextType)
      const ok = window.confirm(
        `Change widget type to "${label}"? Current props will be replaced with defaults for the new type.`,
      )
      if (!ok) return
    }
    emit({
      type: nextType,
      props: defaultPropsForType(nextType),
    })
  }

  function setProps(nextProps: Record<string, unknown>) {
    emit({ props: nextProps })
  }

  function setPropsPatch(nextProps: Record<string, unknown>) {
    emit({ props: nextProps })
  }
</script>

<div class="widget-detail">
  <div class="field-grid">
    <label class="field">
      <span class="label">Type</span>
      {#key widget.type}
        <select value={widget.type} onchange={(e) => changeType(e.currentTarget.value)}>
          {#each WIDGET_TYPE_IDS as typeId (typeId)}
            <option value={typeId}>{widgetTypeLabel(typeId)}</option>
          {/each}
        </select>
      {/key}
    </label>

    <label class="field">
      <span class="label">ID</span>
      <span class="hint-inline">Optional anchor / analytics id.</span>
      <input
        type="text"
        value={widget.id ?? ''}
        spellcheck="false"
        placeholder="(none)"
        oninput={(e) => {
          const raw = e.currentTarget.value
          emit({ id: raw.trim() === '' ? undefined : raw })
        }}
      />
    </label>

    <label class="field checkbox-field">
      <input
        type="checkbox"
        checked={widget.enabled !== false}
        onchange={(e) => {
          const enabled = e.currentTarget.checked
          emit({ enabled: enabled ? undefined : false })
        }}
      />
      Enabled
    </label>
  </div>

  {#if isLayoutWidget(widget.type)}
    <p class="layout-note">
      Layout widgets (<code>row</code>, <code>column</code>, <code>grid</code>) use JSON for
      <code>props.children</code>. Nested structure editing is available in the JSON tab.
    </p>
  {:else if widget.type === 'cover_banner'}
    <CoverBannerPropsForm props={widget.props} onchange={setPropsPatch} />
  {:else if widget.type === 'intro'}
    <IntroPropsForm props={widget.props} onchange={setPropsPatch} />
  {:else if widget.type === 'follow_us'}
    <FollowUsPropsForm props={widget.props} onchange={setPropsPatch} />
  {:else if widget.type === 'info_grid'}
    <InfoGridPropsForm props={widget.props} onchange={setPropsPatch} />
  {:else if widget.type === 'images_grid'}
    <ImagesGridPropsForm props={widget.props} onchange={setPropsPatch} />
  {:else if widget.type === 'media_swiper'}
    <MediaSwiperPropsForm props={widget.props} onchange={setPropsPatch} />
  {:else if widget.type === 'apps_showcase'}
    <AppsShowcasePropsForm props={widget.props} onchange={setPropsPatch} />
  {:else if widget.type === 'careers_tabs'}
    <CareersTabsPropsForm props={widget.props} onchange={setPropsPatch} />
  {:else if widget.type === 'project_grid'}
    <ProjectGridPropsForm props={widget.props} onchange={setPropsPatch} />
  {/if}

  <WidgetPropsJsonFallback props={widget.props} onchange={setProps} />
</div>

<style>
  .widget-detail {
    padding: 0.85rem 0.9rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.4rem;
    background: var(--color-surface-1);
  }

  .field-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr));
    gap: 0.65rem 0.85rem;
    align-items: start;
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

  .hint-inline {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }

  .checkbox-field {
    flex-direction: row;
    align-items: center;
    gap: 0.4rem;
    padding-top: 1.35rem;
    font-size: 0.85rem;
    cursor: pointer;
  }

  select,
  input[type='text'] {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
  }

  input[type='text'] {
    font-family: ui-monospace, Consolas, monospace;
  }

  .layout-note {
    margin: 0.65rem 0 0;
    padding: 0.5rem 0.6rem;
    border-radius: 0.35rem;
    background: var(--color-statusbar-bg);
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .layout-note code {
    font-size: 0.9em;
  }
</style>

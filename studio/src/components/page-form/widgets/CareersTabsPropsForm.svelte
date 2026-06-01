<script lang="ts">
  import { mergeWidgetProps } from '../../../lib/page-form'
  import {
    readCareersLabels,
    readString,
    readVacancies,
    writeCareersLabels,
    writeVacancies,
    type VacancyItem,
  } from '../../../lib/widget-props'
  import StringListField from '../fields/StringListField.svelte'

  type Props = {
    props: Record<string, unknown>
    onchange?: (props: Record<string, unknown>) => void
  }

  let { props, onchange }: Props = $props()

  const labels = $derived(readCareersLabels(props))
  const vacancies = $derived(readVacancies(props))

  function emit(patch: Record<string, unknown>) {
    onchange?.(mergeWidgetProps(props, patch))
  }

  function updateLabels(next: typeof labels) {
    const written = writeCareersLabels(next)
    const base = { ...props }
    if (written) {
      base.labels = written
    } else {
      delete base.labels
    }
    onchange?.(base)
  }

  function updateVacancies(next: VacancyItem[]) {
    emit({ vacancies: writeVacancies(next) })
  }

  function updateVacancy(index: number, patch: Partial<VacancyItem>) {
    updateVacancies(vacancies.map((v, i) => (i === index ? { ...v, ...patch } : v)))
  }

  function addVacancy() {
    updateVacancies([
      ...vacancies,
      { role: '', requirements: [], responsibilities: [], advantages: [], apply_url: '', apply_label: '' },
    ])
  }

  function removeVacancy(index: number) {
    updateVacancies(vacancies.filter((_, i) => i !== index))
  }
</script>

<div class="widget-props-form">
  <label class="field">
    <span class="label">Section title</span>
    <input type="text" value={readString(props, 'title')} oninput={(e) => emit({ title: e.currentTarget.value })} />
  </label>

  <fieldset class="labels-fieldset">
    <legend>Tab labels (optional)</legend>
    <label class="field"><span class="label">Requirements</span><input type="text" value={labels.requirements_title} oninput={(e) => updateLabels({ ...labels, requirements_title: e.currentTarget.value })} /></label>
    <label class="field"><span class="label">Responsibilities</span><input type="text" value={labels.responsibilities_title} oninput={(e) => updateLabels({ ...labels, responsibilities_title: e.currentTarget.value })} /></label>
    <label class="field"><span class="label">Advantages</span><input type="text" value={labels.advantages_title} oninput={(e) => updateLabels({ ...labels, advantages_title: e.currentTarget.value })} /></label>
  </fieldset>

  {#each vacancies as vacancy, index (index)}
    <details class="vacancy-card" open={index === 0}>
      <summary>Vacancy {index + 1}{vacancy.role ? `: ${vacancy.role}` : ''}</summary>
      <div class="card-body">
        <label class="field"><span class="label">Role</span><input type="text" value={vacancy.role} oninput={(e) => updateVacancy(index, { role: e.currentTarget.value })} /></label>
        <StringListField label="Requirements" value={vacancy.requirements} onchange={(requirements) => updateVacancy(index, { requirements })} />
        <StringListField label="Responsibilities" value={vacancy.responsibilities} onchange={(responsibilities) => updateVacancy(index, { responsibilities })} />
        <StringListField label="Advantages" value={vacancy.advantages} onchange={(advantages) => updateVacancy(index, { advantages })} />
        <label class="field"><span class="label">Apply URL</span><input type="text" spellcheck="false" value={vacancy.apply_url} oninput={(e) => updateVacancy(index, { apply_url: e.currentTarget.value })} /></label>
        <label class="field"><span class="label">Apply label</span><input type="text" value={vacancy.apply_label} oninput={(e) => updateVacancy(index, { apply_label: e.currentTarget.value })} /></label>
        <button type="button" class="danger-btn" onclick={() => removeVacancy(index)}>Remove vacancy</button>
      </div>
    </details>
  {/each}

  <button type="button" class="secondary" onclick={addVacancy}>Add vacancy</button>
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

  input[type='text'] {
    width: 100%;
    box-sizing: border-box;
    font-size: 0.82rem;
  }

  .labels-fieldset {
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    padding: 0.55rem 0.65rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .labels-fieldset legend {
    font-size: 0.78rem;
    font-weight: 600;
    padding: 0 0.25rem;
  }

  .vacancy-card {
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.35rem;
    padding: 0.35rem 0.55rem;
    background: var(--color-surface-1);
  }

  .vacancy-card summary {
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

import { Check, Copy, Play, Plus, RotateCcw } from 'lucide-react'
import type { RunDraft } from '../data/demo'
import type { MessageKey, Translate } from '../i18n'

const steps: Array<{ key: string; label: MessageKey }> = [
  { key: 'Source', label: 'source' },
  { key: 'Agent', label: 'agent' },
  { key: 'Environment', label: 'environment' },
  { key: 'Runtime', label: 'runtime' },
  { key: 'Review', label: 'review' },
]

interface RunBuilderProps {
  draft: RunDraft
  activeStep: string
  t: Translate
  onStep: (step: string) => void
  onDraft: (draft: RunDraft) => void
  onLaunch: () => void
}

export function RunBuilder({ draft, activeStep, t, onStep, onDraft, onLaunch }: RunBuilderProps) {
  return (
    <section className="surface run-builder" id="new-run">
      <div className="section-header compact">
        <div>
          <h1>{t('newRun')}</h1>
          <p>{t('newRunDesc')}</p>
        </div>
        <button className="secondary-button">
          <Copy aria-hidden="true" />
          {t('jobConfig')}
        </button>
      </div>
      <div className="step-tabs" role="tablist" aria-label="JobConfig sections">
        {steps.map((step) => (
          <button
            key={step.key}
            className={step.key === activeStep ? 'active' : undefined}
            onClick={() => onStep(step.key)}
          >
            {step.key === activeStep && <Check aria-hidden="true" />}
            {t(step.label)}
          </button>
        ))}
      </div>
      <div className="run-grid">
        <label>
          {t('source')}
          <select value={draft.source} onChange={(event) => onDraft({ ...draft, source: event.target.value })}>
            <option>terminal-bench@2.0</option>
            <option>swe-bench-lite</option>
            <option>harbor/hello-world</option>
          </select>
        </label>
        <label>
          {t('agent')}
          <select value={draft.agent} onChange={(event) => onDraft({ ...draft, agent: event.target.value })}>
            <option>claude-code</option>
            <option>codex-cli</option>
            <option>oracle</option>
          </select>
        </label>
        <label>
          {t('model')}
          <input value={draft.model} onChange={(event) => onDraft({ ...draft, model: event.target.value })} />
        </label>
        <label>
          {t('environment')}
          <select
            value={draft.environment}
            onChange={(event) => onDraft({ ...draft, environment: event.target.value })}
          >
            <option>docker</option>
            <option>local</option>
          </select>
        </label>
        <label>
          {t('concurrency')}
          <input
            type="number"
            min="1"
            value={draft.concurrency}
            onChange={(event) => onDraft({ ...draft, concurrency: Number(event.target.value) })}
          />
        </label>
        <label>
          {t('attempts')}
          <input
            type="number"
            min="1"
            value={draft.attempts}
            onChange={(event) => onDraft({ ...draft, attempts: Number(event.target.value) })}
          />
        </label>
      </div>
      <div className="config-preview">
        <code>
          harbor run --dataset {draft.source} --agent {draft.agent} --model {draft.model} --env {draft.environment}
        </code>
      </div>
      <div className="button-row">
        <button className="secondary-button">
          <RotateCcw aria-hidden="true" />
          {t('reset')}
        </button>
        <button className="secondary-button">
          <Plus aria-hidden="true" />
          {t('saveTemplate')}
        </button>
        <button className="primary-button" onClick={onLaunch}>
          <Play aria-hidden="true" />
          {t('runJob')}
        </button>
      </div>
    </section>
  )
}

import { Copy, Play, Plus, RotateCcw } from 'lucide-react'
import type { RunDraft } from '../data/demo'
import type { Translate } from '../i18n'
import { CustomSelect } from './CustomSelect'

interface RunBuilderProps {
  draft: RunDraft
  t: Translate
  onDraft: (draft: RunDraft) => void
  onLaunch: () => void
}

export function RunBuilder({ draft, t, onDraft, onLaunch }: RunBuilderProps) {
  return (
    <section className="surface run-builder" id="new-job">
      <div className="section-header compact">
        <div>
          <h1>{t('newJob')}</h1>
          <p>{t('newJobDesc')}</p>
        </div>
        <button className="secondary-button">
          <Copy aria-hidden="true" />
          {t('jobConfig')}
        </button>
      </div>
      <div className="run-grid">
        <label>
          {t('source')}
          <CustomSelect
            ariaLabel={t('source')}
            value={draft.source}
            options={[
              { label: 'terminal-bench@2.0', value: 'terminal-bench@2.0' },
              { label: 'swe-bench-lite', value: 'swe-bench-lite' },
              { label: 'harbor/hello-world', value: 'harbor/hello-world' },
            ]}
            onChange={(value) => onDraft({ ...draft, source: value })}
          />
        </label>
        <label>
          {t('agent')}
          <CustomSelect
            ariaLabel={t('agent')}
            value={draft.agent}
            options={[
              { label: 'claude-code', value: 'claude-code' },
              { label: 'codex-cli', value: 'codex-cli' },
              { label: 'oracle', value: 'oracle' },
            ]}
            onChange={(value) => onDraft({ ...draft, agent: value })}
          />
        </label>
        <label>
          {t('model')}
          <input value={draft.model} onChange={(event) => onDraft({ ...draft, model: event.target.value })} />
        </label>
        <label>
          {t('environment')}
          <CustomSelect
            ariaLabel={t('environment')}
            value={draft.environment}
            options={[
              { label: 'docker', value: 'docker' },
              { label: 'local', value: 'local' },
            ]}
            onChange={(value) => onDraft({ ...draft, environment: value })}
          />
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

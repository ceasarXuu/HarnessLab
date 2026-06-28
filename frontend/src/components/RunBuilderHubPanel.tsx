import type { RunDraft } from '../data/demo'
import type { Translate } from '../i18n'
import { CustomSelect } from './CustomSelect'
import { Field, Toggle } from './RunBuilderChrome'

interface RunBuilderHubPanelProps {
  draft: RunDraft
  t: Translate
  onDraft: (draft: RunDraft) => void
}

export function RunBuilderHubPanel({ draft, t, onDraft }: RunBuilderHubPanelProps) {
  return (
    <div className="run-config-groups">
      <section className="run-config-group">
        <div className="run-config-group-heading">
          <h3>{t('artifactMetricSettings')}</h3>
        </div>
        <div className="run-grid">
          <Field label={t('artifacts')}>
            <input value={draft.artifacts} onChange={(event) => onDraft({ ...draft, artifacts: event.target.value })} />
          </Field>
          <Field label={t('metric')}>
            <input value={draft.metric} onChange={(event) => onDraft({ ...draft, metric: event.target.value })} />
          </Field>
        </div>
      </section>

      <section className="run-config-group">
        <div className="run-config-group-heading">
          <h3>{t('jobPlugins')}</h3>
          <p>{t('jobPluginsDesc')}</p>
        </div>
        <div className="plugin-empty-state">{t('noInstalledPlugins')}</div>
        <div className="run-grid">
          <Field label={t('pluginImportPath')}>
            <input value={draft.plugins} onChange={(event) => onDraft({ ...draft, plugins: event.target.value })} />
          </Field>
        </div>
      </section>

      <section className="run-config-group">
        <div className="run-config-group-heading">
          <h3>{t('hubUploadSettings')}</h3>
        </div>
        <div className="run-grid">
          <Field label={t('uploadToHub')}>
            <Toggle checked={draft.upload} onChange={(value) => onDraft({ ...draft, upload: value })} />
          </Field>
          <label>
            {t('visibility')}
            <CustomSelect
              ariaLabel={t('visibility')}
              value={draft.visibility}
              options={[
                { label: 'private', value: 'private' },
                { label: 'public', value: 'public' },
              ]}
              onChange={(value) => onDraft({ ...draft, visibility: value as 'private' | 'public' })}
            />
          </label>
          <Field label={t('shareTargets')}>
            <input value={draft.shareTargets} onChange={(event) => onDraft({ ...draft, shareTargets: event.target.value })} />
          </Field>
        </div>
      </section>
    </div>
  )
}

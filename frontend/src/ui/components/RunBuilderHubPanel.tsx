import type { RunDraft } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { useState } from 'react'
import { CustomSelect } from './CustomSelect'
import { Field, Toggle } from './RunBuilderChrome'

interface RunBuilderHubPanelProps {
  draft: RunDraft
  t: Translate
  onDraft: (draft: RunDraft) => void
}

export function RunBuilderHubPanel({ draft, t, onDraft }: RunBuilderHubPanelProps) {
  const labels = hubLabels(t('runTabHub') === '输出')
  const [advancedOpen, setAdvancedOpen] = useState(false)

  return (
    <div className="run-config-groups">
      <section className="run-config-group">
        <div className="run-config-group-heading">
          <h3>{t('artifacts')}</h3>
        </div>
        <ListControl
          addLabel={labels.add}
          deleteLabel={t('delete')}
          label={t('artifacts')}
          value={draft.artifacts}
          onChange={(value) => onDraft({ ...draft, artifacts: value })}
        />
      </section>

      <section className="run-config-group">
        <div className="run-config-group-heading">
          <h3>{t('hubUploadSettings')}</h3>
        </div>
        <div className="run-grid">
          <Field label={t('uploadToHub')}>
            <Toggle checked={draft.upload} onChange={(value) => onDraft({ ...draft, upload: value })} />
          </Field>
          {draft.upload && (
            <>
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
              <ListControl
                addLabel={labels.add}
                deleteLabel={t('delete')}
                label={t('shareTargets')}
                value={draft.shareTargets}
                onChange={(value) => onDraft({ ...draft, shareTargets: value })}
              />
            </>
          )}
        </div>
      </section>

      <section className="run-config-group">
        <div className="run-config-group-heading">
          <button
            type="button"
            className="runtime-collapsible-trigger"
            aria-expanded={advancedOpen}
            aria-label={advancedOpen ? labels.collapseAdvanced : labels.expandAdvanced}
            onClick={() => setAdvancedOpen((current) => !current)}
          >
            <span className="runtime-collapsible-title">{labels.advanced}</span>
            <span className="runtime-collapsible-icon" aria-hidden="true" />
          </button>
        </div>
        {advancedOpen && (
          <div className="run-grid">
            <label>
              {t('metric')}
              <CustomSelect
                ariaLabel={t('metric')}
                value={draft.metric}
                options={[
                  { label: 'mean', value: 'mean' },
                  { label: 'sum', value: 'sum' },
                  { label: 'min', value: 'min' },
                  { label: 'max', value: 'max' },
                ]}
                onChange={(value) => onDraft({ ...draft, metric: value })}
              />
            </label>
            <section className="field-wide">
              <div className="run-config-group-heading">
                <h3>{t('jobPlugins')}</h3>
              </div>
              <div className="plugin-empty-state">{t('noInstalledPlugins')}</div>
            </section>
          </div>
        )}
      </section>
    </div>
  )
}

function splitList(value: string) {
  return value.split(',').map((item) => item.trim()).filter(Boolean)
}

function formatList(rows: string[]) {
  return rows.map((row) => row.trim()).filter(Boolean).join(',')
}

function ListControl({
  addLabel,
  deleteLabel,
  label,
  onChange,
  value,
}: {
  addLabel: string
  deleteLabel: string
  label: string
  onChange: (value: string) => void
  value: string
}) {
  const [rows, setRows] = useState(() => {
    const items = splitList(value)
    return items.length ? items : ['']
  })
  const commit = (nextRows: string[]) => {
    setRows(nextRows.length ? nextRows : [''])
    onChange(formatList(nextRows))
  }

  return (
    <div className="rule-list-control field-wide">
      <div className="rule-list-header">
        <span>{label}</span>
        <button className="secondary-button compact-action" type="button" onClick={() => setRows([...rows, ''])}>
          {addLabel}
        </button>
      </div>
      <div className="rule-list-rows">
        {rows.map((item, index) => (
          <div className="rule-list-row" key={index}>
            <input
              aria-label={`${label} ${index + 1}`}
              value={item}
              onChange={(event) => commit(rows.map((row, rowIndex) => (rowIndex === index ? event.target.value : row)))}
            />
            <button className="secondary-button compact-action" type="button" onClick={() => commit(rows.filter((_, rowIndex) => rowIndex !== index))}>
              {deleteLabel}
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

function hubLabels(zh: boolean) {
  if (zh) {
    return {
      add: '添加',
      advanced: '高级参数',
      collapseAdvanced: '收起高级参数',
      expandAdvanced: '展开高级参数',
    }
  }
  return {
    add: 'Add',
    advanced: 'Advanced parameters',
    collapseAdvanced: 'Collapse advanced parameters',
    expandAdvanced: 'Expand advanced parameters',
  }
}

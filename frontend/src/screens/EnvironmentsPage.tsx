import { Box, Copy, Plus, Save, Search, Trash2, X } from 'lucide-react'
import { useMemo, useState } from 'react'
import type { EnvironmentRow } from '../mocks/demo'
import type { Translate } from '../i18n'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { TpuSpecControl } from '../ui/components/TpuSpecControl'

type EnvironmentView = 'list' | 'new' | 'copy'
type EnvironmentFieldKind = 'text' | 'select' | 'number' | 'tags' | 'keyValue' | 'json' | 'path' | 'switch' | 'tpu'

interface EnvironmentsPageProps {
  environmentId?: string
  rows: EnvironmentRow[]
  t: Translate
  view: EnvironmentView
  onRowsChange: (rows: EnvironmentRow[]) => void
  onView: (view: EnvironmentView, environmentId?: string) => void
}

interface EnvironmentField {
  key: keyof EnvironmentRow
  label: string
  kind: EnvironmentFieldKind
  options?: string[]
  placeholder?: string
}

interface EnvironmentFieldGroup {
  title: string
  description: string
  fields: EnvironmentField[]
}

const environmentTypes = ['docker', 'e2b', 'daytona', 'modal', 'runloop', 'langsmith', 'gke', 'novita', 'apple-container', 'singularity', 'islo', 'tensorlake', 'cwsandbox', 'wandb', 'use-computer', 'custom']
const networkModes = ['public', 'no-network', 'allowlist']
const operatingSystems = ['linux', 'windows']
const resourcePolicies = ['auto', 'limit', 'request', 'guarantee', 'ignore']

const environmentFieldGroups: EnvironmentFieldGroup[] = [
  {
    title: 'OrnnLab template',
    description: '本地模板信息，用于在 New Job 中复用，不是 Harbor 原生资源。',
    fields: [
      { key: 'name', label: 'Environment Name', kind: 'text' },
      { key: 'environmentType', label: 'type', kind: 'select', options: environmentTypes },
      { key: 'importPath', label: 'import_path', kind: 'path', placeholder: 'module.path:ClassName' },
    ],
  },
  {
    title: 'Task environment baseline',
    description: '映射 task manifest 的 [environment] 字段。',
    fields: [
      { key: 'dockerImage', label: 'docker_image', kind: 'text', placeholder: 'python:3.13-slim' },
      { key: 'os', label: 'os', kind: 'select', options: operatingSystems },
      { key: 'networkMode', label: 'network_mode', kind: 'select', options: networkModes },
      { key: 'allowedHosts', label: 'allowed_hosts', kind: 'tags', placeholder: 'pypi.org, github.com' },
      { key: 'cpus', label: 'cpus', kind: 'number' },
      { key: 'memoryMb', label: 'memory_mb', kind: 'number' },
      { key: 'storageMb', label: 'storage_mb', kind: 'number' },
      { key: 'gpus', label: 'gpus', kind: 'number' },
      { key: 'gpuTypes', label: 'gpu_types', kind: 'tags', placeholder: 'A100, H100' },
      { key: 'tpu', label: 'tpu', kind: 'tpu' },
      { key: 'env', label: 'env', kind: 'keyValue', placeholder: 'KEY=value' },
      { key: 'skillsDir', label: 'skills_dir', kind: 'path' },
      { key: 'healthcheck', label: 'healthcheck', kind: 'json' },
      { key: 'workdir', label: 'workdir', kind: 'path' },
    ],
  },
  {
    title: 'Runtime overrides',
    description: '映射 Job/Trial EnvironmentConfig 的运行时覆盖项。',
    fields: [
      { key: 'forceBuild', label: 'force_build', kind: 'switch' },
      { key: 'deleteAfterRun', label: 'delete', kind: 'switch' },
      { key: 'cpuPolicy', label: 'cpu_enforcement_policy', kind: 'select', options: resourcePolicies },
      { key: 'memoryPolicy', label: 'memory_enforcement_policy', kind: 'select', options: resourcePolicies },
      { key: 'overrideCpus', label: 'override_cpus', kind: 'number' },
      { key: 'overrideMemoryMb', label: 'override_memory_mb', kind: 'number' },
      { key: 'overrideStorageMb', label: 'override_storage_mb', kind: 'number' },
      { key: 'overrideGpus', label: 'override_gpus', kind: 'number' },
      { key: 'overrideTpu', label: 'override_tpu', kind: 'tpu' },
      { key: 'mounts', label: 'mounts', kind: 'json' },
      { key: 'dockerCompose', label: 'extra_docker_compose', kind: 'path' },
      { key: 'extraAllowedHosts', label: 'extra_allowed_hosts', kind: 'tags', placeholder: 'model.internal' },
      { key: 'kwargs', label: 'kwargs', kind: 'keyValue' },
    ],
  },
]

export function EnvironmentsPage({ environmentId, rows, t, view, onRowsChange, onView }: EnvironmentsPageProps) {
  const [selected, setSelected] = useState<EnvironmentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<EnvironmentRow | null>(null)
  const [editingDraft, setEditingDraft] = useState<EnvironmentRow | null>(null)
  const [search, setSearch] = useState('')
  const filteredRows = useMemo(() => {
    const query = search.trim().toLowerCase()
    if (!query) return rows
    return rows.filter((row) =>
      [row.name, row.profileType, row.environmentType, row.importPath, row.dockerImage, row.networkMode, row.allowedHosts].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [rows, search])

  const openDrawer = (row: EnvironmentRow) => {
    setSelected(row)
    setDrawerOpen(true)
    setEditingDraft({ ...row })
  }

  const saveNewTemplate = (draft: EnvironmentRow) => {
    const saved = { ...draft, id: buildEnvironmentId(rows, draft.name), profileType: 'custom' as const }
    onRowsChange([...rows, saved])
    setSelected(saved)
    setEditingDraft(saved)
    setDrawerOpen(true)
    onView('list')
  }

  const saveDrawerEdit = () => {
    if (!editingDraft) return
    const saved = { ...editingDraft, name: editingDraft.name.trim() || 'Custom Environment' }
    onRowsChange(rows.map((row) => (row.id === saved.id ? saved : row)))
    setSelected(saved)
    setEditingDraft(saved)
  }

  const confirmDelete = () => {
    if (!deleteTarget) return
    const nextRows = rows.filter((row) => row.id !== deleteTarget.id)
    onRowsChange(nextRows)
    if (selected?.id === deleteTarget.id) {
      setSelected(null)
      setDrawerOpen(false)
      setEditingDraft(null)
    }
    setDeleteTarget(null)
  }

  if (view !== 'list') {
    const source = rows.find((row) => row.id === environmentId)
    const initialValue =
      view === 'copy' && source
        ? { ...source, id: buildEnvironmentId(rows, source.name), name: `${source.name} copy`, profileType: 'custom' as const }
        : buildNewEnvironment(rows)
    return (
      <EnvironmentFormPage
        key={`${view}-${environmentId ?? 'new'}`}
        initialValue={initialValue}
        title={view === 'copy' ? t('copyEnvironment') : t('newEnvironment')}
        t={t}
        onCancel={() => onView('list')}
        onSave={saveNewTemplate}
      />
    )
  }

  return (
    <main className="workspace single-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('environmentsCatalog')}</h1>
          </div>
          <div className="toolbar">
            <label className="search-field">
              <Search aria-hidden="true" />
              <input
                aria-label={t('searchEnvironments')}
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder={t('searchEnvironmentsPlaceholder')}
              />
            </label>
            <button className="primary-button" onClick={() => onView('new')}>
              <Plus aria-hidden="true" />
              {t('newEnvironment')}
            </button>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('environmentName')}</th>
                <th>profile</th>
                <th>type</th>
                <th>docker_image</th>
                <th>network_mode</th>
                <th>cpu / memory policy</th>
                <th>runtime overrides</th>
                <th>{t('actions')}</th>
              </tr>
            </thead>
            <tbody>
              {filteredRows.map((row) => (
                <tr
                  key={row.id}
                  className={selected?.id === row.id ? 'selected-row' : undefined}
                  onClick={() => openDrawer(row)}
                >
                  <td>
                    <span className="cell-title">
                      <Box aria-hidden="true" />
                      {row.name}
                    </span>
                  </td>
                  <td>{row.profileType}</td>
                  <td>{row.environmentType}</td>
                  <td>{row.dockerImage}</td>
                  <td>{row.networkMode}</td>
                  <td>{row.cpuPolicy} / {row.memoryPolicy}</td>
                  <td>{formatOverrides(row)}</td>
                  <td>
                    <EnvironmentActions
                      row={row}
                      t={t}
                      onCopy={(target) => onView('copy', target.id)}
                      onDelete={setDeleteTarget}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
      {selected && (
        <DetailDrawer label={t('selectedEnvironment')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <aside className="detail-rail">
            <section className="surface rail-card">
              <div className="rail-heading">
                <div>
                  <h2>{selected.name}</h2>
                  <p>{selected.environmentType}</p>
                </div>
                <div className="row-actions">
                  <button className="primary-button compact-action" onClick={saveDrawerEdit}>
                    <Save aria-hidden="true" />
                    {t('save')}
                  </button>
                  <EnvironmentActions
                    row={selected}
                    t={t}
                    onCopy={(target) => onView('copy', target.id)}
                    onDelete={setDeleteTarget}
                  />
                </div>
              </div>
              {editingDraft && <EnvironmentProfileEditor value={editingDraft} onChange={setEditingDraft} />}
            </section>
          </aside>
        </DetailDrawer>
      )}
      {deleteTarget && (
        <div className="confirm-overlay">
          <section className="surface confirm-dialog" role="dialog" aria-modal="true" aria-label={t('deleteEnvironmentTitle')}>
            <div className="confirm-heading">
              <h2>{t('deleteEnvironmentTitle')}</h2>
            </div>
            <ul className="cleanup-impact-list">
              <li>{t('deleteEnvironmentLocalImpact')}</li>
              <li>{deleteTarget.name}</li>
            </ul>
            <div className="button-row confirm-actions">
              <button className="secondary-button" onClick={() => setDeleteTarget(null)}>{t('cancel')}</button>
              <button className="primary-button" onClick={confirmDelete}>{t('confirmDelete')}</button>
            </div>
          </section>
        </div>
      )}
    </main>
  )
}

function EnvironmentFormPage({
  initialValue,
  title,
  t,
  onCancel,
  onSave,
}: {
  initialValue: EnvironmentRow
  title: string
  t: Translate
  onCancel: () => void
  onSave: (value: EnvironmentRow) => void
}) {
  const [draft, setDraft] = useState(initialValue)
  return (
    <main className="workspace single-page">
      <div className="content-column">
        <nav className="breadcrumb-nav" aria-label="Environment path">
          <button type="button" onClick={onCancel}>
            {t('environments')}
          </button>
          <span aria-current="page">{title}</span>
        </nav>
        <section className="surface run-builder">
          <div className="section-header compact">
            <div>
              <h1>{title}</h1>
            </div>
            <div className="run-builder-actions">
              <button className="secondary-button" onClick={onCancel}>
                <X aria-hidden="true" />
                {t('cancel')}
              </button>
              <button className="primary-button" onClick={() => onSave(draft)}>
                <Save aria-hidden="true" />
                {t('save')}
              </button>
            </div>
          </div>
          <EnvironmentProfileEditor value={draft} onChange={setDraft} />
        </section>
      </div>
    </main>
  )
}

function EnvironmentProfileEditor({ value, onChange }: { value: EnvironmentRow; onChange: (value: EnvironmentRow) => void }) {
  return (
    <div className="environment-editor">
      {environmentFieldGroups.map((group) => (
        <section className="run-config-group" key={group.title}>
          {group.title !== 'OrnnLab template' && (
            <div className="run-config-group-heading">
              <h3>{group.title}</h3>
            </div>
          )}
          <div className="run-grid">
            {group.fields.filter((field) => isEnvironmentFieldVisible(field, value)).map((field) => (
              <EnvironmentFieldControl
                field={field}
                key={field.key}
                value={value}
                onChange={onChange}
              />
            ))}
          </div>
        </section>
      ))}
    </div>
  )
}

function EnvironmentFieldControl({
  field,
  value,
  onChange,
}: {
  field: EnvironmentField
  value: EnvironmentRow
  onChange: (value: EnvironmentRow) => void
}) {
  const currentValue = value[field.key]
  const setValue = (nextValue: string | boolean) => onChange({ ...value, [field.key]: nextValue })
  if (field.kind === 'switch') {
    return (
      <label className="switch-control environment-switch">
        <span>{field.label}</span>
        <input
          checked={Boolean(currentValue)}
          onChange={(event) => setValue(event.target.checked)}
          type="checkbox"
        />
      </label>
    )
  }
  if (field.kind === 'select') {
    return (
      <label>
        {field.label}
        <select value={String(currentValue)} onChange={(event) => setValue(event.target.value)}>
          {field.options?.map((option) => (
            <option key={option} value={option}>{option}</option>
          ))}
        </select>
      </label>
    )
  }
  if (field.kind === 'json' || field.kind === 'keyValue') {
    return (
      <label className="field-wide">
        {field.label}
        <textarea
          placeholder={field.placeholder}
          value={String(currentValue)}
          onChange={(event) => setValue(event.target.value)}
        />
      </label>
    )
  }
  if (field.kind === 'tpu') {
    return (
      <TpuSpecControl
        label={field.label}
        value={String(currentValue)}
        onChange={setValue}
      />
    )
  }
  const displayValue = field.kind === 'number' ? normalizeNumberValue(currentValue) : normalizeInputValue(currentValue)
  return (
    <label>
      {field.label}
      <input
        inputMode={field.kind === 'number' ? 'numeric' : undefined}
        placeholder={field.placeholder}
        type={field.kind === 'number' ? 'number' : 'text'}
        value={displayValue}
        onChange={(event) => setValue(event.target.value)}
      />
    </label>
  )
}

function isEnvironmentFieldVisible(field: EnvironmentField, value: EnvironmentRow) {
  if (field.key === 'importPath') return value.environmentType === 'custom'
  if (field.key === 'allowedHosts') return value.networkMode === 'allowlist'
  return true
}

function normalizeInputValue(value: EnvironmentRow[keyof EnvironmentRow]) {
  return typeof value === 'boolean' ? String(value) : String(value).replace(/^none$/, '')
}

function normalizeNumberValue(value: EnvironmentRow[keyof EnvironmentRow]) {
  const text = normalizeInputValue(value)
  return /^-?\d+(\.\d+)?$/.test(text) ? text : ''
}

function EnvironmentActions({
  row,
  t,
  onCopy,
  onDelete,
}: {
  row: EnvironmentRow
  t: Translate
  onCopy: (row: EnvironmentRow) => void
  onDelete: (row: EnvironmentRow) => void
}) {
  return (
    <div className="row-actions" onClick={(event) => event.stopPropagation()}>
      <button className="secondary-button compact-action" onClick={() => onCopy(row)}>
        <Copy aria-hidden="true" />
        {t('copy')}
      </button>
      {row.profileType === 'custom' && (
        <button className="secondary-button compact-action" onClick={() => onDelete(row)}>
          <Trash2 aria-hidden="true" />
          {t('delete')}
        </button>
      )}
    </div>
  )
}

function buildNewEnvironment(rows: EnvironmentRow[]): EnvironmentRow {
  return {
    ...rows[0],
    id: buildEnvironmentId(rows, 'Custom Environment'),
    name: 'Custom Environment',
    profileType: 'custom',
    importPath: 'none',
    dockerImage: 'task environment image',
  }
}

function buildEnvironmentId(rows: EnvironmentRow[], name: string) {
  const base = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'environment'
  let candidate = base
  let index = 2
  const existing = new Set(rows.map((row) => row.id))
  while (existing.has(candidate)) {
    candidate = `${base}-${index}`
    index += 1
  }
  return candidate
}

function formatOverrides(row: EnvironmentRow) {
  const values = [row.overrideCpus, row.overrideMemoryMb, row.overrideStorageMb, row.overrideGpus, row.overrideTpu]
  return values.some((value) => value !== 'none') ? values.filter((value) => value !== 'none').join(' / ') : 'none'
}

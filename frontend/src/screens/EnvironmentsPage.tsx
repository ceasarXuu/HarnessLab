import { Box, Copy, Plus, Save, Search, Trash2, X } from 'lucide-react'
import { useMemo, useState } from 'react'
import type { EnvironmentRow } from '../mocks/demo'
import type { Translate } from '../i18n'
import { DetailDrawer } from '../ui/components/DetailDrawer'

type EnvironmentView = 'list' | 'new' | 'copy'

interface EnvironmentsPageProps {
  environmentId?: string
  rows: EnvironmentRow[]
  t: Translate
  view: EnvironmentView
  onRowsChange: (rows: EnvironmentRow[]) => void
  onView: (view: EnvironmentView, environmentId?: string) => void
}

const editableFields: Array<{ key: keyof EnvironmentRow; label: string }> = [
  { key: 'name', label: 'Environment Name' },
  { key: 'environmentType', label: 'type' },
  { key: 'importPath', label: 'import_path' },
  { key: 'networkMode', label: 'network_mode' },
  { key: 'allowedHosts', label: 'allowed_hosts' },
  { key: 'dockerImage', label: 'docker_image' },
  { key: 'cpus', label: 'cpus' },
  { key: 'memoryMb', label: 'memory_mb' },
  { key: 'storageMb', label: 'storage_mb' },
  { key: 'gpus', label: 'gpus' },
  { key: 'env', label: 'env' },
  { key: 'kwargs', label: 'kwargs' },
  { key: 'mounts', label: 'mounts' },
  { key: 'dockerCompose', label: 'extra_docker_compose' },
]

const editableDetailFields: Array<{ key: keyof EnvironmentRow; label: string }> = [
  { key: 'name', label: 'Environment Name' },
  { key: 'environmentType', label: 'type' },
  { key: 'importPath', label: 'import_path' },
  { key: 'networkMode', label: 'network_mode' },
  { key: 'allowedHosts', label: 'allowed_hosts' },
  { key: 'dockerImage', label: 'docker_image' },
  { key: 'os', label: 'os' },
  { key: 'cpus', label: 'cpus' },
  { key: 'memoryMb', label: 'memory_mb' },
  { key: 'storageMb', label: 'storage_mb' },
  { key: 'gpus', label: 'gpus' },
  { key: 'gpuTypes', label: 'gpu_types' },
  { key: 'tpu', label: 'tpu' },
  { key: 'skillsDir', label: 'skills_dir' },
  { key: 'healthcheck', label: 'healthcheck' },
  { key: 'workdir', label: 'workdir' },
  { key: 'mounts', label: 'mounts' },
  { key: 'env', label: 'env' },
  { key: 'kwargs', label: 'kwargs' },
  { key: 'cpuPolicy', label: 'cpu_enforcement_policy' },
  { key: 'memoryPolicy', label: 'memory_enforcement_policy' },
  { key: 'overrideCpus', label: 'override_cpus' },
  { key: 'overrideMemoryMb', label: 'override_memory_mb' },
  { key: 'overrideStorageMb', label: 'override_storage_mb' },
  { key: 'overrideGpus', label: 'override_gpus' },
  { key: 'overrideTpu', label: 'override_tpu' },
  { key: 'dockerCompose', label: 'extra_docker_compose' },
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
              {editingDraft && <EnvironmentFields value={editingDraft} fields={editableDetailFields} onChange={setEditingDraft} />}
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
          <EnvironmentFields value={draft} onChange={setDraft} />
        </section>
      </div>
    </main>
  )
}

function EnvironmentFields({
  fields = editableFields,
  value,
  onChange,
}: {
  fields?: Array<{ key: keyof EnvironmentRow; label: string }>
  value: EnvironmentRow
  onChange: (value: EnvironmentRow) => void
}) {
  return (
    <div className="run-grid">
      {fields.map((field) => (
        <label key={field.key}>
          {field.label}
          <input
            value={String(value[field.key])}
            onChange={(event) => onChange({ ...value, [field.key]: event.target.value })}
          />
        </label>
      ))}
    </div>
  )
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

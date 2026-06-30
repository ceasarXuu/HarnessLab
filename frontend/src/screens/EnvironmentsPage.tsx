import { Box, Copy, Pencil, Plus, Search, Trash2 } from 'lucide-react'
import { useMemo, useState } from 'react'
import type { EnvironmentRow } from '../mocks/demo'
import type { Translate } from '../i18n'
import { DetailDrawer } from '../ui/components/DetailDrawer'

interface EnvironmentsPageProps {
  rows: EnvironmentRow[]
  t: Translate
  onRowsChange: (rows: EnvironmentRow[]) => void
}

type FormMode = 'new' | 'edit' | 'copy'

interface FormState {
  mode: FormMode
  value: EnvironmentRow
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

const detailFields: Array<{ key: keyof EnvironmentRow; label: string }> = [
  { key: 'name', label: 'Environment Name' },
  { key: 'profileType', label: 'profile' },
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

export function EnvironmentsPage({ rows, t, onRowsChange }: EnvironmentsPageProps) {
  const [selected, setSelected] = useState<EnvironmentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<EnvironmentRow | null>(null)
  const [form, setForm] = useState<FormState | null>(null)
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
  }

  const openNew = () => {
    setForm({ mode: 'new', value: buildNewEnvironment(rows) })
  }

  const openCopy = (row: EnvironmentRow) => {
    setForm({ mode: 'copy', value: { ...row, id: buildEnvironmentId(rows, row.name), name: `${row.name} copy`, profileType: 'custom' } })
  }

  const openEdit = (row: EnvironmentRow) => {
    if (row.profileType === 'built-in') return
    setForm({ mode: 'edit', value: { ...row } })
  }

  const saveForm = () => {
    if (!form) return
    const nextValue = { ...form.value, name: form.value.name.trim() || 'Custom Environment', profileType: 'custom' as const }
    const savedValue = form.mode === 'edit' ? nextValue : { ...nextValue, id: buildEnvironmentId(rows, nextValue.name) }
    const nextRows =
      form.mode === 'edit'
        ? rows.map((row) => (row.id === savedValue.id ? savedValue : row))
        : [...rows, savedValue]
    onRowsChange(nextRows)
    setSelected(savedValue)
    setDrawerOpen(true)
    setForm(null)
  }

  const confirmDelete = () => {
    if (!deleteTarget) return
    const nextRows = rows.filter((row) => row.id !== deleteTarget.id)
    onRowsChange(nextRows)
    if (selected?.id === deleteTarget.id) {
      setSelected(null)
      setDrawerOpen(false)
    }
    setDeleteTarget(null)
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
            <button className="primary-button" onClick={openNew}>
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
                      onCopy={openCopy}
                      onDelete={setDeleteTarget}
                      onEdit={openEdit}
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
                <EnvironmentActions
                  row={selected}
                  t={t}
                  onCopy={openCopy}
                  onDelete={setDeleteTarget}
                  onEdit={openEdit}
                />
              </div>
              <div className="metric-grid">
                {detailFields.map((field) => (
                  <Metric key={field.key} label={field.label} value={String(selected[field.key])} />
                ))}
                <Metric label="force_build" value={selected.forceBuild ? 'true' : 'false'} />
                <Metric label="delete" value={selected.deleteAfterRun ? 'true' : 'false'} />
              </div>
            </section>
          </aside>
        </DetailDrawer>
      )}
      {form && (
        <EnvironmentFormDialog
          form={form}
          t={t}
          onCancel={() => setForm(null)}
          onChange={(value) => setForm({ ...form, value })}
          onSave={saveForm}
        />
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

function EnvironmentActions({
  row,
  t,
  onCopy,
  onDelete,
  onEdit,
}: {
  row: EnvironmentRow
  t: Translate
  onCopy: (row: EnvironmentRow) => void
  onDelete: (row: EnvironmentRow) => void
  onEdit: (row: EnvironmentRow) => void
}) {
  return (
    <div className="row-actions" onClick={(event) => event.stopPropagation()}>
      <button className="secondary-button compact-action" onClick={() => onCopy(row)}>
        <Copy aria-hidden="true" />
        {t('copy')}
      </button>
      {row.profileType === 'custom' && (
        <>
          <button className="secondary-button compact-action" onClick={() => onEdit(row)}>
            <Pencil aria-hidden="true" />
            {t('edit')}
          </button>
          <button className="secondary-button compact-action" onClick={() => onDelete(row)}>
            <Trash2 aria-hidden="true" />
            {t('delete')}
          </button>
        </>
      )}
    </div>
  )
}

function EnvironmentFormDialog({
  form,
  t,
  onCancel,
  onChange,
  onSave,
}: {
  form: FormState
  t: Translate
  onCancel: () => void
  onChange: (value: EnvironmentRow) => void
  onSave: () => void
}) {
  const title = form.mode === 'edit' ? t('editEnvironment') : form.mode === 'copy' ? t('copyEnvironment') : t('newEnvironment')
  return (
    <div className="confirm-overlay">
      <section className="surface confirm-dialog" role="dialog" aria-modal="true" aria-label={title}>
        <div className="confirm-heading">
          <h2>{title}</h2>
        </div>
        <div className="import-dataset-form">
          {editableFields.map((field) => (
            <label key={field.key}>
              {field.label}
              <input
                value={String(form.value[field.key])}
                onChange={(event) => onChange({ ...form.value, [field.key]: event.target.value })}
              />
            </label>
          ))}
        </div>
        <div className="button-row confirm-actions">
          <button className="secondary-button" onClick={onCancel}>{t('cancel')}</button>
          <button className="primary-button" onClick={onSave}>{t('save')}</button>
        </div>
      </section>
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

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  )
}

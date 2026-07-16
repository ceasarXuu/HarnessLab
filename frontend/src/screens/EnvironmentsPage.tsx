import { Box, Copy, Plus, Save, Search, Trash2, X } from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useCachedServerSearch, useEnvironment, useOperation } from '../api/hooks'
import { environmentRowToDto } from '../api/requestMappers'
import { environmentDtoToRow } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { defaultEnvironmentDraft } from '../domain/defaults'
import type { EnvironmentRow } from '../domain/harbor'
import type { Translate } from '../i18n'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { EnvironmentProfileEditor } from '../ui/components/EnvironmentProfileEditor'
import { Pagination } from '../ui/components/Pagination'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import { FormValidationSummary, issuesByField, type FormIssue } from '../ui/components/FormValidationSummary'
import { usePaginatedItems } from '../ui/pagination'

type EnvironmentView = 'list' | 'new' | 'copy'

interface EnvironmentsPageProps {
  writesEnabled?: boolean
  client: WebUiClient
  environmentId?: string
  rows: EnvironmentRow[]
  t: Translate
  view: EnvironmentView
  onRefresh: () => Promise<void>
  onView: (view: EnvironmentView, environmentId?: string) => void
}

export function EnvironmentsPage({ writesEnabled = true, client, environmentId, rows, t, view, onRefresh, onView }: EnvironmentsPageProps) {
  const [selected, setSelected] = useState<EnvironmentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<EnvironmentRow | null>(null)
  const [editingDraft, setEditingDraft] = useState<EnvironmentRow | null>(null)
  const [search, setSearch] = useState('')
  const searchQuery = search.trim() || undefined
  const loadSearch = useCallback((query: string) => client.listEnvironments({ limit: 100, q: query }), [client])
  const searchResource = useCachedServerSearch('environments', searchQuery, loadSearch)
  const detailResource = useEnvironment(client, selected?.id)
  const environmentOperation = useOperation(client)
  const detailEnvironment = detailResource.data ? environmentDtoToRow(detailResource.data) : selected

  useEffect(() => {
    if (!detailResource.data) return
    const next = environmentDtoToRow(detailResource.data)
    setSelected(next)
    setEditingDraft((current) => (current?.id === next.id ? next : current))
  }, [detailResource.data])

  useEffect(() => {
    if (environmentOperation.operation?.status !== 'completed') return
    void onRefresh()
    void detailResource.refresh()
    if (environmentOperation.operation.type === 'create-environment') onView('list')
  }, [detailResource.refresh, environmentOperation.operation?.id, environmentOperation.operation?.status, onRefresh, onView])
  const filteredRows = useMemo(() => {
    if (!searchQuery) return rows
    if (searchResource.data) return searchResource.data.items.map(environmentDtoToRow)
    const query = searchQuery.toLowerCase()
    return rows.filter((row) =>
      [row.name, row.profileType, row.environmentType, row.importPath, row.allowedHosts].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [rows, searchQuery, searchResource.data])
  const pagination = usePaginatedItems({ items: filteredRows, resetKey: search })

  const openDrawer = (row: EnvironmentRow) => {
    setSelected(row)
    setDrawerOpen(true)
    setEditingDraft({ ...row })
  }

  const saveNewTemplate = async (draft: EnvironmentRow) => {
    if (!writesEnabled) return
    const saved = { ...draft, id: buildEnvironmentId(rows, draft.name), profileType: 'custom' as const }
    await environmentOperation.submit(() => client.createEnvironment(environmentRowToDto(saved)), ({ operation }) => operation)
  }

  const saveDrawerEdit = async () => {
    if (!writesEnabled) return
    if (!editingDraft) return
    const saved = { ...editingDraft, name: editingDraft.name.trim() || 'Custom Environment' }
    await environmentOperation.submit(() => client.updateEnvironment(saved.id, environmentRowToDto(saved)), ({ operation }) => operation)
  }

  const confirmDelete = async () => {
    if (!writesEnabled) return
    if (!deleteTarget) return
    await environmentOperation.submit(() => client.deleteEnvironment(deleteTarget.id), ({ operation }) => operation)
    if (selected?.id === deleteTarget.id) {
      setSelected(null)
      setDrawerOpen(false)
      setEditingDraft(null)
    }
    setDeleteTarget(null)
  }

  const copyEnvironment = async (row: EnvironmentRow) => {
    if (!writesEnabled) return
    await environmentOperation.submit(() => client.copyEnvironment(row.id), ({ operation }) => operation)
  }

  if (view !== 'list') {
    const source = rows.find((row) => row.id === environmentId)
    const initialValue =
      view === 'copy' && source
        ? { ...source, id: buildEnvironmentId(rows, source.name), name: `${source.name} copy`, profileType: 'custom' as const }
        : buildNewEnvironment(rows)
    return (
      <>
        <EnvironmentFormPage
          key={`${view}-${environmentId ?? 'new'}`}
          initialValue={initialValue}
          canSave={writesEnabled && !isOperationRunning(environmentOperation.operation?.status)}
          serverError={environmentOperation.error?.message}
          title={view === 'copy' ? t('copyEnvironment') : t('newEnvironment')}
          t={t}
          onCancel={() => onView('list')}
          onSave={saveNewTemplate}
        />
      </>
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
                aria-busy={searchResource.loading}
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder={t('searchEnvironmentsPlaceholder')}
              />
            </label>
            <button className="primary-button" disabled={!writesEnabled} onClick={() => onView('new')}>
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
                <th>{t('environmentProfile')}</th>
                <th>{t('agentType')}</th>
                <th>{t('actions')}</th>
              </tr>
            </thead>
            <tbody>
              {pagination.items.map((row) => (
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
                  <td>
                    <EnvironmentActions
                      row={row}
                      disabled={!writesEnabled || isOperationRunning(environmentOperation.operation?.status)}
                      t={t}
                      onCopy={copyEnvironment}
                      onDelete={setDeleteTarget}
                    />
                  </td>
                </tr>
              ))}
              {filteredRows.length === 0 && (
                <tr>
                  <td className="empty-row" colSpan={4}>{t('noEnvironmentsAvailable')}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
        <Pagination {...pagination} t={t} onPage={pagination.setPage} />
      </section>
      <ResourceStatus error={searchResource.error?.message ?? null} />
      {detailEnvironment && (
        <DetailDrawer label={t('selectedEnvironment')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <>
            <aside className="detail-rail">
              <section className="surface rail-card">
                <div className="rail-heading">
                  <div>
                    <h2>{detailEnvironment.name}</h2>
                    <p>{detailEnvironment.environmentType}</p>
                  </div>
                  <div className="row-actions">
                    <button className="primary-button compact-action" disabled={!writesEnabled || isOperationRunning(environmentOperation.operation?.status)} onClick={saveDrawerEdit}>
                      <Save aria-hidden="true" />
                      {t('save')}
                    </button>
                    <EnvironmentActions
                      row={detailEnvironment}
                      disabled={!writesEnabled || isOperationRunning(environmentOperation.operation?.status)}
                      t={t}
                      onCopy={copyEnvironment}
                      onDelete={setDeleteTarget}
                    />
                  </div>
                </div>
                {editingDraft && <EnvironmentProfileEditor value={editingDraft} t={t} onChange={setEditingDraft} />}
              </section>
            </aside>
            <ResourceStatus
              error={detailResource.error?.message ?? null}
              loading={detailResource.loading}
              loadingLabel={t('loadingEnvironments')}
            />
          </>
        </DetailDrawer>
      )}
      {deleteTarget && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={t('confirmDelete')}
          impacts={[t('deleteEnvironmentLocalImpact'), deleteTarget.name]}
          title={t('deleteEnvironmentTitle')}
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      )}
      <ResourceStatus error={environmentOperation.error?.message ?? null} />
    </main>
  )
}

function EnvironmentFormPage({
  initialValue,
  canSave,
  title,
  t,
  onCancel,
  onSave,
  serverError,
}: {
  initialValue: EnvironmentRow
  canSave: boolean
  title: string
  t: Translate
  onCancel: () => void
  onSave: (value: EnvironmentRow) => void
  serverError?: string | null
}) {
  const [draft, setDraft] = useState(initialValue)
  const [validationAttempted, setValidationAttempted] = useState(false)
  const allIssues = validateEnvironment(draft, t)
  const issues = validationAttempted ? allIssues : []
  const fieldErrors = issuesByField(issues)
  const submit = () => {
    setValidationAttempted(true)
    if (allIssues.length) {
      window.requestAnimationFrame(() => document.querySelector<HTMLElement>('.form-validation-summary')?.focus())
      return
    }
    onSave(draft)
  }
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
              <button className="primary-button" disabled={!canSave} onClick={submit}>
                <Save aria-hidden="true" />
                {t('save')}
              </button>
            </div>
          </div>
          <FormValidationSummary
            issues={issues}
            serverError={serverError}
            title={t('formValidationTitle')}
            onIssue={(field) => document.getElementById(`environment-${field}`)?.focus()}
          />
          <EnvironmentProfileEditor fieldErrors={fieldErrors} value={draft} t={t} onChange={setDraft} />
        </section>
      </div>
    </main>
  )
}

function validateEnvironment(draft: EnvironmentRow, t: Translate): FormIssue[] {
  const issues: FormIssue[] = []
  if (!draft.name.trim()) issues.push({ field: 'name', message: t('environmentNameRequired') })
  if (!draft.environmentType.trim()) issues.push({ field: 'environmentType', message: t('environmentTypeRequired') })
  return issues
}

function EnvironmentActions({
  row,
  disabled,
  t,
  onCopy,
  onDelete,
}: {
  row: EnvironmentRow
  disabled: boolean
  t: Translate
  onCopy: (row: EnvironmentRow) => void
  onDelete: (row: EnvironmentRow) => void
}) {
  return (
    <div className="row-actions" onClick={(event) => event.stopPropagation()}>
      <button className="secondary-button compact-action" disabled={disabled} onClick={() => onCopy(row)}>
        <Copy aria-hidden="true" />
        {t('copy')}
      </button>
      {row.profileType === 'custom' && (
        <button className="secondary-button compact-action" disabled={disabled} onClick={() => onDelete(row)}>
          <Trash2 aria-hidden="true" />
          {t('delete')}
        </button>
      )}
    </div>
  )
}

function isOperationRunning(status: string | undefined) {
  return status === 'queued' || status === 'running'
}

function buildNewEnvironment(rows: EnvironmentRow[]): EnvironmentRow {
  return {
    ...defaultEnvironmentDraft,
    id: buildEnvironmentId(rows, 'Custom Environment'),
    name: 'Custom Environment',
    profileType: 'custom',
    importPath: 'none',
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

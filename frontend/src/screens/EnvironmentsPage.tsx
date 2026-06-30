import { Box, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import type { EnvironmentRow } from '../mocks/demo'
import type { Translate } from '../i18n'
import { DetailDrawer } from '../ui/components/DetailDrawer'

interface EnvironmentsPageProps {
  rows: EnvironmentRow[]
  t: Translate
}

export function EnvironmentsPage({ rows, t }: EnvironmentsPageProps) {
  const [environmentRows, setEnvironmentRows] = useState(rows)
  const [selected, setSelected] = useState<EnvironmentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<EnvironmentRow | null>(null)
  const [search, setSearch] = useState('')
  const filteredRows = useMemo(() => {
    const query = search.trim().toLowerCase()
    if (!query) return environmentRows
    return environmentRows.filter((row) =>
      [row.name, row.backend, row.type, row.image, row.resources, row.status].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [environmentRows, search])

  const confirmDelete = () => {
    if (!deleteTarget) return
    setEnvironmentRows((current) => current.filter((row) => row.id !== deleteTarget.id))
    if (selected?.id === deleteTarget.id) {
      setDrawerOpen(false)
      setSelected(null)
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
            <button className="primary-button">{t('newEnvironment')}</button>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('environmentName')}</th>
                <th>{t('backend')}</th>
                <th>{t('agentType')}</th>
                <th>{t('image')}</th>
                <th>{t('resourcePolicy')}</th>
                <th>{t('status')}</th>
                <th>{t('actions')}</th>
              </tr>
            </thead>
            <tbody>
              {filteredRows.map((row) => (
                <tr
                  key={row.id}
                  className={selected?.id === row.id ? 'selected-row' : undefined}
                  onClick={() => {
                    setSelected(row)
                    setDrawerOpen(true)
                  }}
                >
                  <td>
                    <span className="cell-title">
                      <Box aria-hidden="true" />
                      {row.name}
                    </span>
                  </td>
                  <td>{row.backend}</td>
                  <td>{row.type}</td>
                  <td>{row.image}</td>
                  <td>{row.resources}</td>
                  <td>
                    <span className={`status-dot ${row.status === 'needs-review' ? 'warning' : 'success'}`}>
                      {row.status}
                    </span>
                  </td>
                  <td>
                    <div className="row-actions">
                      {row.type === 'custom' && (
                        <button className="secondary-button compact-action" onClick={(event) => {
                          event.stopPropagation()
                          setDeleteTarget(row)
                        }}>
                          {t('delete')}
                        </button>
                      )}
                    </div>
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
                  <p>{selected.backend}</p>
                </div>
                <span className={`status-dot ${selected.status === 'needs-review' ? 'warning' : 'success'}`}>
                  {selected.status}
                </span>
              </div>
              <div className="metric-grid">
                <Metric label={t('environmentName')} value={selected.name} />
                <Metric label={t('backend')} value={selected.backend} />
                <Metric label={t('agentType')} value={selected.type} />
                <Metric label={t('image')} value={selected.image} />
                <Metric label={t('resourcePolicy')} value={selected.resources} />
                <Metric label={t('mounts')} value={selected.mounts} />
                <Metric label="env" value={selected.env} />
                <Metric label="allowed hosts" value={selected.allowedHosts} />
                <Metric label={t('forceBuild')} value={selected.forceBuild ? 'enabled' : 'disabled'} />
                <Metric label={t('deleteEnvironment')} value={selected.deleteAfterRun ? 'enabled' : 'disabled'} />
                <Metric label={t('sourceRef')} value={selected.source} />
                <Metric label={t('updated')} value={selected.updated} />
              </div>
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

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  )
}

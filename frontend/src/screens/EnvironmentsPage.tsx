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
  const [selected, setSelected] = useState<EnvironmentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [search, setSearch] = useState('')
  const filteredRows = useMemo(() => {
    const query = search.trim().toLowerCase()
    if (!query) return rows
    return rows.filter((row) =>
      [row.name, row.environmentType, row.importPath, row.dockerImage, row.networkMode, row.allowedHosts].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [rows, search])

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
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('environmentName')}</th>
                <th>type</th>
                <th>docker_image</th>
                <th>network_mode</th>
                <th>cpu / memory policy</th>
                <th>runtime overrides</th>
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
                  <td>{row.environmentType}</td>
                  <td>{row.dockerImage}</td>
                  <td>{row.networkMode}</td>
                  <td>{row.cpuPolicy} / {row.memoryPolicy}</td>
                  <td>{formatOverrides(row)}</td>
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
              </div>
              <div className="metric-grid">
                <Metric label={t('environmentName')} value={selected.name} />
                <Metric label="type" value={selected.environmentType} />
                <Metric label="import_path" value={selected.importPath} />
                <Metric label="network_mode" value={selected.networkMode} />
                <Metric label="allowed_hosts" value={selected.allowedHosts} />
                <Metric label="docker_image" value={selected.dockerImage} />
                <Metric label="os" value={selected.os} />
                <Metric label="cpus" value={selected.cpus} />
                <Metric label="memory_mb" value={selected.memoryMb} />
                <Metric label="storage_mb" value={selected.storageMb} />
                <Metric label="gpus" value={selected.gpus} />
                <Metric label="gpu_types" value={selected.gpuTypes} />
                <Metric label="tpu" value={selected.tpu} />
                <Metric label="skills_dir" value={selected.skillsDir} />
                <Metric label="healthcheck" value={selected.healthcheck} />
                <Metric label="workdir" value={selected.workdir} />
                <Metric label={t('mounts')} value={selected.mounts} />
                <Metric label="env" value={selected.env} />
                <Metric label="kwargs" value={selected.kwargs} />
                <Metric label="force_build" value={selected.forceBuild ? 'true' : 'false'} />
                <Metric label="delete" value={selected.deleteAfterRun ? 'true' : 'false'} />
                <Metric label="cpu_enforcement_policy" value={selected.cpuPolicy} />
                <Metric label="memory_enforcement_policy" value={selected.memoryPolicy} />
                <Metric label="override_cpus" value={selected.overrideCpus} />
                <Metric label="override_memory_mb" value={selected.overrideMemoryMb} />
                <Metric label="override_storage_mb" value={selected.overrideStorageMb} />
                <Metric label="override_gpus" value={selected.overrideGpus} />
                <Metric label="override_tpu" value={selected.overrideTpu} />
                <Metric label="extra_docker_compose" value={selected.dockerCompose} />
              </div>
            </section>
          </aside>
        </DetailDrawer>
      )}
    </main>
  )
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

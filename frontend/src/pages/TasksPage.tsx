import { Box } from 'lucide-react'
import type { TaskRow } from '../data/demo'
import type { Translate } from '../i18n'

interface TasksPageProps {
  rows: TaskRow[]
  t: Translate
}

export function TasksPage({ rows, t }: TasksPageProps) {
  return (
    <main className="workspace single-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('taskQueue')}</h1>
            <p>{t('taskQueueDesc')}</p>
          </div>
          <div className="toolbar">
            <button className="secondary-button">{t('retry')}</button>
            <button className="primary-button">{t('runJob')}</button>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('taskName')}</th>
                <th>{t('job')}</th>
                <th>{t('os')}</th>
                <th>{t('status')}</th>
                <th>{t('duration')}</th>
                <th>{t('owner')}</th>
                <th>{t('verifier')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.name}>
                  <td>
                    <span className="cell-title">
                      <Box aria-hidden="true" />
                      {row.name}
                    </span>
                  </td>
                  <td>{row.jobId}</td>
                  <td>{row.os}</td>
                  <td>
                    <span className={`status-dot ${row.state === 'ok' ? 'success' : row.state}`}>{row.state}</span>
                  </td>
                  <td>{row.duration}</td>
                  <td>{row.owner}</td>
                  <td>{row.verifier}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  )
}

import { FlaskConical } from 'lucide-react'
import type { TrialRow } from '../data/demo'
import type { Translate } from '../i18n'

interface TrialsPageProps {
  rows: TrialRow[]
  t: Translate
}

export function TrialsPage({ rows, t }: TrialsPageProps) {
  return (
    <main className="workspace single-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('trialMatrix')}</h1>
            <p>{t('trialMatrixDesc')}</p>
          </div>
          <div className="toolbar">
            <button className="secondary-button">{t('artifactPaths')}</button>
            <button className="primary-button">{t('retry')}</button>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('trial')}</th>
                <th>{t('job')}</th>
                <th>{t('taskName')}</th>
                <th>{t('result')}</th>
                <th>{t('score')}</th>
                <th>{t('retries')}</th>
                <th>{t('duration')}</th>
                <th>{t('logPath')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.id}>
                  <td>
                    <span className="cell-title">
                      <FlaskConical aria-hidden="true" />
                      {row.id}
                    </span>
                  </td>
                  <td>{row.jobId}</td>
                  <td>{row.task}</td>
                  <td>
                    <span className={`status-dot ${row.result === 'passed' ? 'success' : row.result}`}>
                      {row.result}
                    </span>
                  </td>
                  <td>{row.score}</td>
                  <td>{row.retries}</td>
                  <td>{row.duration}</td>
                  <td>
                    <code>{row.logPath}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  )
}

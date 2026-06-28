import { CheckCircle2, ServerCog, TerminalSquare } from 'lucide-react'
import type { SystemRow } from '../data/demo'
import type { Translate } from '../i18n'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

export function SystemPage({ rows, t }: SystemPageProps) {
  const commandGroups = [
    {
      title: t('authCommands'),
      commands: [
        { label: t('status'), command: 'harbor auth status' },
        { label: t('login'), command: 'harbor auth login' },
        { label: t('logout'), command: 'harbor auth logout' },
      ],
    },
    {
      title: t('cacheCommands'),
      commands: [
        { label: t('cache'), command: 'harbor cache clean --dry-run' },
        { label: t('plugins'), command: 'harbor plugins list' },
      ],
    },
    {
      title: t('manifestCommands'),
      commands: [
        { label: t('sync'), command: 'harbor sync ./dataset.toml' },
      ],
    },
    {
      title: t('hubCommands'),
      commands: [
        { label: t('upload'), command: 'harbor upload jobs/job_91a7 --private --share-user @ornn' },
        { label: t('submit'), command: 'harbor leaderboard submit job_91a7' },
        { label: t('share'), command: 'harbor job share job_91a7 --share-org ornn' },
      ],
    },
  ]

  return (
    <main className="workspace two-column-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('systemHealth')}</h1>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('component')}</th>
                <th>{t('status')}</th>
                <th>{t('value')}</th>
                <th>{t('evidence')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.component}>
                  <td>
                    <span className="cell-title">
                      <ServerCog aria-hidden="true" />
                      {row.component}
                    </span>
                  </td>
                  <td>
                    <span className={`status-dot ${row.status}`}>{row.status}</span>
                  </td>
                  <td>{row.value}</td>
                  <td>
                    <code>{row.evidence}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
      <aside className="detail-rail">
        <section className="surface rail-card">
          <div className="rail-title">
            <CheckCircle2 aria-hidden="true" />
            <h3>{t('systemChecks')}</h3>
          </div>
          <ul className="doctor-list">
            <li>
              <CheckCircle2 aria-hidden="true" />
              {t('runsReconciled')}
            </li>
            <li>
              <CheckCircle2 aria-hidden="true" />
              {t('artifactStoreWritable')}
            </li>
            <li>
              <CheckCircle2 aria-hidden="true" />
              {t('cacheCommandAvailable')}
            </li>
          </ul>
        </section>
        <section className="surface rail-card">
          <div className="rail-title">
            <TerminalSquare aria-hidden="true" />
            <h3>{t('harborCommands')}</h3>
          </div>
          <div className="command-groups">
            {commandGroups.map((group) => (
              <section key={group.title} className="command-group">
                <h4>{group.title}</h4>
                <div className="command-list">
                  {group.commands.map((command) => (
                    <div key={command.command} className="command-row">
                      <button className="secondary-button">{command.label}</button>
                      <code>{command.command}</code>
                    </div>
                  ))}
                </div>
              </section>
            ))}
          </div>
        </section>
      </aside>
    </main>
  )
}

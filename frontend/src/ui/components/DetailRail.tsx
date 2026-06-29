import { useState } from 'react'
import { Download, FileJson, FlaskConical, Play, Terminal, Upload } from 'lucide-react'
import type { EventLog, HarborJob, TrialRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'

interface DetailRailProps {
  job: HarborJob
  events: EventLog[]
  trials: TrialRow[]
  t: Translate
}

export function DetailRail({ job, events, trials, t }: DetailRailProps) {
  const [expandedTrialId, setExpandedTrialId] = useState<string | null>(null)

  return (
    <aside className="detail-rail">
      <section className="surface rail-card job-summary-card">
        <div className="rail-heading">
          <div className="rail-title-copy">
            <h2>{job.name}</h2>
            <p>{job.dataset}</p>
          </div>
          <span className={`status-dot ${job.status}`}>{job.status}</span>
        </div>
        <div className="metric-grid">
          <Metric label={t('trialCount')} value={job.trials} />
          <Metric label={t('score')} value={job.score} />
          <Metric label={t('cost')} value={job.cost} />
          <Metric label="tokens" value={job.tokens} />
          <Metric label={t('environment')} value={job.environment} />
          <Metric label="job_dir" value={job.jobDir ?? 'jobs/current'} />
          <Metric label="split" value={job.split ?? 'default'} />
        </div>
        <div className="button-row tight job-action-row">
          <button className="secondary-button">
            <Play aria-hidden="true" />
            {t('resume')}
          </button>
          <button className="secondary-button">{t('openViewer')}</button>
          <button className="secondary-button">
            <Upload aria-hidden="true" />
            {t('upload')}
          </button>
          <button className="secondary-button">
            <Download aria-hidden="true" />
            {t('download')}
          </button>
        </div>
      </section>

      <section className="surface rail-card">
        <div className="rail-title">
          <FlaskConical aria-hidden="true" />
          <h3>{t('jobTrials')}</h3>
        </div>
        <div className="mini-table">
          <div className="mini-row trial-row mini-header" role="row">
            <span>{t('taskName')}</span>
            <span>{t('result')}</span>
            <span>{t('duration')}</span>
            <span>{t('cost')} / tokens</span>
          </div>
          {trials.map((trial) => (
            <div key={trial.id} className="trial-entry">
              <button
                type="button"
                className="mini-row trial-row trial-toggle"
                aria-expanded={expandedTrialId === trial.id}
                onClick={() => setExpandedTrialId((current) => (current === trial.id ? null : trial.id))}
              >
                <span>{trial.task}</span>
                <span className={`status-dot ${trial.result === 'passed' ? 'success' : trial.result}`}>{trial.result}</span>
                <span>{trial.duration}</span>
                <span>{trial.cost} / {trial.tokens}</span>
              </button>
              {expandedTrialId === trial.id && (
                <div className="trial-expanded">
                  <code>retries: {trial.retries}</code>
                  <code>log path: {trial.logPath}</code>
                </div>
              )}
            </div>
          ))}
        </div>
      </section>

      <section className="surface rail-card">
        <div className="rail-title">
          <Terminal aria-hidden="true" />
          <h3>{t('eventLog')}</h3>
        </div>
        <ol className="event-list">
          {events.map((event) => (
            <li key={`${event.time}-${event.message}`} className={event.level}>
              <span>{event.time}</span>
              <p>{event.message}</p>
            </li>
          ))}
        </ol>
      </section>

      <section className="surface rail-card">
        <div className="rail-title">
          <FileJson aria-hidden="true" />
          <h3>{t('artifactPaths')}</h3>
        </div>
        <div className="path-list">
          <code>harbor.config.json</code>
          <code>harbor.capability.json</code>
          <code>result.json</code>
          <code>job.log</code>
          <code>{job.jobDir ?? `jobs/${job.id}`}</code>
          <code>trials/{job.id}</code>
          {job.failureCode && <code>{job.failureCode}</code>}
        </div>
      </section>
    </aside>
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

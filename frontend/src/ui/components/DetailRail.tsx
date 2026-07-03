import { useState } from 'react'
import { FileJson, FlaskConical, Pause, Play, Terminal, Upload } from 'lucide-react'
import type { EventLog, HarborJob, TrialRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { Metric } from './Metric'

interface DetailRailProps {
  job: HarborJob
  events: EventLog[]
  trials: TrialRow[]
  t: Translate
  onLeaderboardChange: (jobId: string, include: boolean) => void
}

export function DetailRail({ job, events, trials, t, onLeaderboardChange }: DetailRailProps) {
  const [expandedTrialId, setExpandedTrialId] = useState<string | null>(null)
  const artifactPaths = job.artifactPaths ?? buildArtifactPaths(job)
  const primaryJobAction = getPrimaryJobAction(job.status, t)

  return (
    <aside className="detail-rail">
      <section className="surface rail-card job-summary-card">
        <div className="rail-heading">
          <div className="rail-title-copy">
            <h2>{job.name}</h2>
            <p>{job.dataset}</p>
          </div>
          <div className="rail-heading-actions">
            <span className={`status-dot ${job.status}`}>{job.status}</span>
            <button className="secondary-button compact-button">
              {primaryJobAction.kind === 'pause' ? <Pause aria-hidden="true" /> : <Play aria-hidden="true" />}
              {primaryJobAction.label}
            </button>
          </div>
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
          <label className="switch-control">
            <span>{t('includeInLeaderboard')}</span>
            <input
              type="checkbox"
              checked={job.includeInLeaderboard}
              onChange={(event) => onLeaderboardChange(job.id, event.target.checked)}
            />
          </label>
          <button className="secondary-button">{t('openViewer')}</button>
          <button className="secondary-button">
            <Upload aria-hidden="true" />
            {t('upload')}
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
        <p className="rail-subtitle">
          <span>{t('eventLogPath')}</span>
          <code>{job.eventLogPath ?? `${job.jobDir ?? `jobs/${job.id}`}/job.log`}</code>
        </p>
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
          {artifactPaths.map((path) => (
            <code key={path}>{path}</code>
          ))}
        </div>
      </section>
    </aside>
  )
}

function getPrimaryJobAction(status: HarborJob['status'], t: Translate) {
  if (status === 'running' || status === 'queued') {
    return { kind: 'pause' as const, label: t('pause') }
  }

  return { kind: 'resume' as const, label: t('resume') }
}

function buildArtifactPaths(job: HarborJob) {
  const root = getJobRootPath(job)
  const paths = [
    `${root}/harbor.config.json`,
    `${root}/harbor.capability.json`,
    `${root}/result.json`,
    `${root}/job.log`,
    root,
    `/Users/xuzhang/.ornnlab/HarnessLab/trials/${job.id}`,
  ]

  if (job.failureCode) {
    paths.push(`${root}/${job.failureCode}`)
  }

  return paths
}

function getJobRootPath(job: HarborJob) {
  if (job.eventLogPath?.endsWith('/job.log')) {
    return job.eventLogPath.slice(0, -'/job.log'.length)
  }

  return `/Users/xuzhang/.ornnlab/HarnessLab/${job.jobDir ?? `jobs/${job.id}`}`
}

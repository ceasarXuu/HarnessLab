import { useState } from 'react'
import { FileJson, FlaskConical, RotateCcw, Square, Terminal } from 'lucide-react'
import type { EventLog, HarborJob, TrialRow } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { JobStatusBadge } from './JobStatusBadge'
import { Metric } from './Metric'
import { SwitchControl } from './SwitchControl'

interface DetailRailProps {
  writesEnabled?: boolean
  job: HarborJob
  events: EventLog[]
  trials: TrialRow[]
  t: Translate
  onJobAction: (jobId: string, action: 'cancel' | 'resume') => void
  onLeaderboardChange: (jobId: string, include: boolean) => void
}

export function DetailRail({ writesEnabled = true, job, events, trials, t, onJobAction, onLeaderboardChange }: DetailRailProps) {
  const [expandedTrialId, setExpandedTrialId] = useState<string | null>(null)
  const artifactPaths = job.artifactPaths ?? buildArtifactPaths(job)
  const primaryJobAction = getPrimaryJobAction(job, t)

  return (
    <aside className="detail-rail">
      <section className="surface rail-card job-summary-card">
        <div className="rail-heading">
          <div className="rail-title-copy">
            <h2>{job.name}</h2>
            <p>{job.dataset}</p>
          </div>
          <div className="rail-heading-actions">
            <JobStatusBadge status={job.status} t={t} />
            {primaryJobAction && (
              <button
                className="secondary-button compact-button"
                disabled={!writesEnabled}
                onClick={() => onJobAction(job.id, primaryJobAction.kind)}
              >
                {primaryJobAction.kind === 'cancel' ? <Square aria-hidden="true" /> : <RotateCcw aria-hidden="true" />}
                {primaryJobAction.label}
              </button>
            )}
          </div>
        </div>
        <div className="metric-grid">
          <Metric label={t('trialCount')} value={job.trials} />
          <Metric label={t('score')} value={job.score} />
          <Metric label={t('cost')} value={job.cost} />
          <Metric label="tokens" value={job.tokens} />
          <Metric label={t('environment')} value={job.environment} />
          <Metric label="job_dir" value={job.jobDir ?? 'jobs/current'} />
        </div>
        <div className="button-row tight job-action-row">
          <SwitchControl
            checked={job.includeInLeaderboard}
            disabled={!writesEnabled}
            label={t('includeInLeaderboard')}
            onChange={(checked) => onLeaderboardChange(job.id, checked)}
          />
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

function getPrimaryJobAction(job: HarborJob, t: Translate) {
  if (job.status === 'running' || job.status === 'queued') {
    return { kind: 'cancel' as const, label: t('cancel') }
  }
  if ((job.status === 'failed' || job.status === 'interrupted') && job.canResume) {
    return { kind: 'resume' as const, label: t('resume') }
  }
  return null
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

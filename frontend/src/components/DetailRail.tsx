import { AlertTriangle, CheckCircle2, FileJson, FlaskConical, Square, Terminal } from 'lucide-react'
import type { EventLog, HarborJob, TrialRow } from '../data/demo'
import type { Translate } from '../i18n'

interface DetailRailProps {
  job: HarborJob
  events: EventLog[]
  trials: TrialRow[]
  t: Translate
}

export function DetailRail({ job, events, trials, t }: DetailRailProps) {
  return (
    <aside className="detail-rail">
      <section className="surface rail-card">
        <p className="panel-kicker">{t('selectedJob')}</p>
        <div className="rail-heading">
          <div>
            <h2>{job.name}</h2>
            <p>{job.dataset}</p>
          </div>
          <span className={`status-dot ${job.status}`}>{job.status}</span>
        </div>
        <div className="metric-grid">
          <Metric label={t('trialCount')} value={job.trials} />
          <Metric label={t('score')} value={job.score} />
          <Metric label={t('cost')} value={job.cost} />
          <Metric label={t('environment')} value={job.environment} />
        </div>
        <div className="button-row tight">
          <button className="secondary-button">
            <Square aria-hidden="true" />
            {t('cancel')}
          </button>
          <button className="secondary-button">{t('retry')}</button>
        </div>
      </section>

      <section className="surface rail-card">
        <div className="rail-title">
          <FlaskConical aria-hidden="true" />
          <h3>{t('jobTrials')}</h3>
        </div>
        <div className="mini-table">
          {trials.map((trial) => (
            <div key={trial.id} className="mini-row trial-row">
              <span>{trial.task}</span>
              <span className={`status-dot ${trial.result === 'passed' ? 'success' : trial.result}`}>{trial.result}</span>
              <span>{trial.score}</span>
              <span>{trial.duration}</span>
              <span>{trial.cost}</span>
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
          <CheckCircle2 aria-hidden="true" />
          <h3>{t('systemDoctor')}</h3>
        </div>
        <ul className="doctor-list">
          <li>
            <CheckCircle2 aria-hidden="true" />
            Harbor 0.13.x available
          </li>
          <li>
            <CheckCircle2 aria-hidden="true" />
            Docker context colima
          </li>
          <li>
            <AlertTriangle aria-hidden="true" />
            1 verifier retry
          </li>
        </ul>
      </section>

      <section className="surface rail-card">
        <div className="rail-title">
          <FileJson aria-hidden="true" />
          <h3>{t('artifactPaths')}</h3>
        </div>
        <div className="path-list">
          <code>harbor.config.json</code>
          <code>result.json</code>
          <code>job.log</code>
          <code>trials/{job.id}</code>
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

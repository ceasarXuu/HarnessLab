import { AlertTriangle, Box, CheckCircle2, FileJson, Square, Terminal } from 'lucide-react'
import type { EventLog, HarborJob } from '../data/demo'

interface DetailRailProps {
  job: HarborJob
  events: EventLog[]
}

export function DetailRail({ job, events }: DetailRailProps) {
  return (
    <aside className="detail-rail">
      <section className="surface rail-card">
        <div className="rail-heading">
          <div>
            <h2>{job.name}</h2>
            <p>{job.dataset}</p>
          </div>
          <span className={`status-dot ${job.status}`}>{job.status}</span>
        </div>
        <div className="metric-grid">
          <Metric label="Trials" value={job.trials} />
          <Metric label="Score" value={job.score} />
          <Metric label="Cost" value={job.cost} />
          <Metric label="Env" value={job.environment} />
        </div>
        <div className="button-row tight">
          <button className="secondary-button">
            <Square aria-hidden="true" />
            Cancel
          </button>
          <button className="secondary-button">Retry</button>
        </div>
      </section>

      <section className="surface rail-card">
        <div className="rail-title">
          <Terminal aria-hidden="true" />
          <h3>Event log</h3>
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
          <h3>System doctor</h3>
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
          <h3>Artifact paths</h3>
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

export function TaskPreview({ rows }: { rows: string[][] }) {
  return (
    <section className="surface task-preview" id="tasks">
      <div className="rail-title">
        <Box aria-hidden="true" />
        <h2>Tasks</h2>
      </div>
      <div className="mini-table">
        {rows.map(([name, os, state, duration]) => (
          <div key={name} className="mini-row">
            <span>{name}</span>
            <span>{os}</span>
            <span>{state}</span>
            <span>{duration}</span>
          </div>
        ))}
      </div>
    </section>
  )
}

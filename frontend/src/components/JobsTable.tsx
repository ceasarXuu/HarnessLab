import { Search } from 'lucide-react'
import type { HarborJob, JobStatus } from '../data/demo'

const statusLabels: Record<JobStatus, string> = {
  running: 'Running',
  queued: 'Queued',
  completed: 'Completed',
  failed: 'Failed',
}

interface JobsTableProps {
  jobs: HarborJob[]
  selectedId: string
  search: string
  onSearch: (value: string) => void
  onSelect: (job: HarborJob) => void
}

export function JobsTable({ jobs, selectedId, search, onSearch, onSelect }: JobsTableProps) {
  return (
    <section className="surface jobs-surface" id="jobs">
      <div className="section-header">
        <div>
          <h1>Jobs</h1>
          <p>Local Harbor runs, status, results, and recovery evidence.</p>
        </div>
        <div className="toolbar">
          <label className="search-field">
            <Search aria-hidden="true" />
            <input
              aria-label="Search jobs"
              value={search}
              onChange={(event) => onSearch(event.target.value)}
              placeholder="Search jobs, datasets, agents"
            />
          </label>
          <button className="secondary-button">Import</button>
          <button className="primary-button">New Run</button>
        </div>
      </div>
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Job</th>
              <th>Status</th>
              <th>Dataset</th>
              <th>Agent</th>
              <th>Model</th>
              <th>Trials</th>
              <th>Score</th>
              <th>Cost</th>
              <th>Updated</th>
            </tr>
          </thead>
          <tbody>
            {jobs.map((job) => (
              <tr
                key={job.id}
                className={job.id === selectedId ? 'selected-row' : undefined}
                onClick={() => onSelect(job)}
              >
                <td>
                  <button className="row-button">{job.name}</button>
                  <small>{job.id}</small>
                </td>
                <td>
                  <span className={`status-dot ${job.status}`}>{statusLabels[job.status]}</span>
                </td>
                <td>{job.dataset}</td>
                <td>{job.agent}</td>
                <td>{job.model}</td>
                <td>{job.trials}</td>
                <td>{job.score}</td>
                <td>{job.cost}</td>
                <td>{job.updated}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  )
}

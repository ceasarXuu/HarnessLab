import { Search } from 'lucide-react'
import type { HarborJob, JobStatus } from '../../domain/harbor'
import type { Translate } from '../../i18n'

const statusLabels: Record<JobStatus, string> = {
  running: 'Running',
  queued: 'Queued',
  completed: 'Completed',
  failed: 'Failed',
}

interface JobsTableProps {
  jobs: HarborJob[]
  selectedId?: string
  search: string
  t: Translate
  onNewJob: () => void
  onSearch: (value: string) => void
  onSelect: (job: HarborJob) => void
}

export function JobsTable({ jobs, selectedId, search, t, onNewJob, onSearch, onSelect }: JobsTableProps) {
  return (
    <section className="surface jobs-surface" id="jobs">
      <div className="section-header">
        <div>
          <h1>{t('jobRegistry')}</h1>
        </div>
        <div className="toolbar">
          <label className="search-field">
            <Search aria-hidden="true" />
            <input
              aria-label={t('searchJobs')}
              value={search}
              onChange={(event) => onSearch(event.target.value)}
              placeholder={t('searchJobsPlaceholder')}
            />
          </label>
          <button className="primary-button" onClick={onNewJob}>
            {t('newJob')}
          </button>
        </div>
      </div>
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>{t('job')}</th>
              <th>{t('status')}</th>
              <th>{t('dataset')}</th>
              <th>{t('agent')}</th>
              <th>{t('model')}</th>
              <th>{t('trialCount')}</th>
              <th>{t('score')}</th>
              <th>{t('cost')}</th>
              <th>{t('tokenUsage')}</th>
              <th>{t('runtimeDuration')}</th>
              <th>{t('createdTime')}</th>
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
                  <span className="job-identity">
                    <button className="row-button">{job.name}</button>
                    <small>{job.id}</small>
                  </span>
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
                <td>{job.tokenUsage}</td>
                <td>{job.runtimeDuration}</td>
                <td>{job.createdAt}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  )
}

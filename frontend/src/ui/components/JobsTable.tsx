import { LoaderCircle, Search } from 'lucide-react'
import type { HarborJob } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import type { PaginationState } from '../pagination'
import { Pagination } from './Pagination'

interface JobsTableProps {
  jobs: HarborJob[]
  pagination?: PaginationState<HarborJob>
  selectedId?: string
  search: string
  t: Translate
  onNewJob: () => void
  onSearch: (value: string) => void
  onSelect: (job: HarborJob) => void
}

export function JobsTable({ jobs, pagination, selectedId, search, t, onNewJob, onSearch, onSelect }: JobsTableProps) {
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
              <th>{t('dataset')}</th>
              <th>{t('agent')}</th>
              <th>{t('model')}</th>
              <th>{t('taskTotal')}</th>
              <th>{t('taskCompleted')}</th>
              <th>{t('taskErrored')}</th>
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
                    <span className="job-name-line">
                      {job.status === 'running' && (
                        <LoaderCircle className="job-running-spinner" aria-label={t('statusRunning')} />
                      )}
                      <button className="row-button" onClick={() => onSelect(job)}>{job.name}</button>
                    </span>
                    <small>{job.id}</small>
                  </span>
                </td>
                <td>{job.dataset}</td>
                <td>{job.agent}</td>
                <td>{job.model}</td>
                <td>{job.trial.total}</td>
                <td>
                  <span className="job-completed-counts">
                    <span>{t('taskPassed')} {job.trial.passed}</span>
                    <span>{t('taskNotPassed')} {job.trial.notPassed}</span>
                  </span>
                </td>
                <td>{job.trial.errored}</td>
                <td>{job.score}</td>
                <td>{job.cost}</td>
                <td>{job.tokenUsage}</td>
                <td>{job.runtimeDuration}</td>
                <td>{job.createdAt}</td>
              </tr>
            ))}
            {jobs.length === 0 && (
              <tr>
                <td className="empty-row" colSpan={12}>{t('noJobsAvailable')}</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      {pagination && <Pagination {...pagination} t={t} onPage={pagination.setPage} />}
    </section>
  )
}

import { act, fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { TrialRow } from '../../domain/harbor'
import { getTranslator } from '../../i18n'
import { jobs } from '../../mocks/demo'
import { DetailRail } from './DetailRail'

describe('DetailRail Job actions', () => {
  it('only offers resume when the failed Job has Harbor resume artifacts', () => {
    const failed = jobs.find((job) => job.status === 'failed')!
    const { rerender } = render(
      <DetailRail
        job={{ ...failed, canResume: false }}
        events={[]}
        trials={[]}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
      />,
    )

    expect(screen.queryByRole('button', { name: 'Resume' })).not.toBeInTheDocument()

    rerender(
      <DetailRail
        job={{ ...failed, canResume: true }}
        events={[]}
        trials={[]}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
      />,
    )
    expect(screen.getByRole('button', { name: 'Resume' })).toBeVisible()
  })

  it('renders in-progress trials as running rows without fabricated values', () => {
    const running = jobs.find((job) => job.status === 'running')!
    const trials: TrialRow[] = [
      {
        analysisPath: '', artifactPath: '', cost: '-', duration: '-', id: 'running-trial',
        jobId: running.id, logPath: '/tmp/trial.log', progress: 'running', result: 'running',
        error: '', retries: 0, score: '-', task: 'build-cython-ext', tokens: '-', verifierEvidence: '',
      },
      {
        analysisPath: '', artifactPath: '', cost: '$0.50', duration: '00:05:00', id: 'passed-trial',
        jobId: running.id, logPath: '/tmp/other.log', progress: 'passed', result: 'passed',
        error: '', retries: 0, score: '1.0/1.0', task: 'sqlite-with-gcov', tokens: '0.1M', verifierEvidence: '',
      },
    ]

    render(
      <DetailRail
        job={running}
        events={[]}
        trials={trials}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
      />,
    )

    const runningRow = screen.getByRole('button', { name: /build-cython-ext/ })
    expect(runningRow).toHaveTextContent('running')
    expect(runningRow).toHaveTextContent('-')
    const passedRow = screen.getByRole('button', { name: /sqlite-with-gcov/ })
    expect(passedRow).toHaveTextContent('passed')
    expect(passedRow).toHaveTextContent('00:05:00')
  })

  it('renders pending tasks with a muted pending status', () => {
    const job = jobs[0]
    const trials: TrialRow[] = [
      {
        analysisPath: '', artifactPath: '', cost: '-', duration: '-', id: 'pending-task',
        jobId: job.id, logPath: '', progress: 'pending', result: 'pending',
        error: '', retries: 0, score: '-', task: 'configure-git-webserver', tokens: '-', verifierEvidence: '',
      },
    ]

    render(
      <DetailRail
        job={job}
        events={[]}
        trials={trials}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
      />,
    )

    const row = screen.getByRole('button', { name: /configure-git-webserver/ })
    expect(row).toHaveTextContent('pending')
    expect(row.querySelector('.status-dot')).toHaveClass('pending')
  })

  it('renders the trial failure reason in the expanded detail', async () => {
    const job = jobs[0]
    const trials: TrialRow[] = [
      {
        analysisPath: '', artifactPath: '', cost: '-', duration: '-', error: 'NonZeroAgentExitCodeError: Command failed (exit 1)',
        id: 'failed-trial', jobId: job.id, logPath: '/tmp/f.log', progress: 'errored', result: 'errored',
        retries: 0, score: '-', task: 'configure-git-webserver', tokens: '-', verifierEvidence: '',
      },
    ]

    render(
      <DetailRail
        job={job}
        events={[]}
        trials={trials}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: /configure-git-webserver/ }))
    expect(await screen.findByText(/NonZeroAgentExitCodeError: Command failed/)).toBeInTheDocument()
  })

  it('shows re-run failed tasks only for terminal jobs with errored trials', () => {
    const job = jobs[0]
    const onRerunFailed = vi.fn()
    const errored: TrialRow[] = [
      {
        analysisPath: '', artifactPath: '', cost: '-', duration: '-', error: 'NonZeroAgentExitCodeError',
        id: 'e1', jobId: job.id, logPath: '', progress: 'errored', result: 'errored',
        retries: 0, score: '-', task: 'hello-world', tokens: '-', verifierEvidence: '',
      },
    ]

    const { rerender } = render(
      <DetailRail
        job={{ ...job, status: 'completed' }}
        events={[]}
        trials={errored}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
        onRerunFailed={onRerunFailed}
      />,
    )
    const button = screen.getByRole('button', { name: 'Re-run failed tasks' })
    expect(button).toBeInTheDocument()
    fireEvent.click(button)
    expect(onRerunFailed).toHaveBeenCalledWith(expect.objectContaining({ id: job.id }))

    rerender(
      <DetailRail
        job={{ ...job, status: 'running' }}
        events={[]}
        trials={errored}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
        onRerunFailed={onRerunFailed}
      />,
    )
    expect(screen.queryByRole('button', { name: 'Re-run failed tasks' })).not.toBeInTheDocument()

    rerender(
      <DetailRail
        job={{ ...job, status: 'completed' }}
        events={[]}
        trials={[]}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
        onRerunFailed={onRerunFailed}
      />,
    )
    expect(screen.queryByRole('button', { name: 'Re-run failed tasks' })).not.toBeInTheDocument()
  })

  it('ticks a live timer for running tasks', async () => {
    vi.useFakeTimers()
    try {
      const job = jobs[0]
      const startedAt = new Date(Date.now() - 65_000).toISOString()
      const trials: TrialRow[] = [
        {
          analysisPath: '', artifactPath: '', cost: '-', duration: '-', error: '',
          id: 'running-timer', jobId: job.id, logPath: '', progress: 'running', result: 'running',
          retries: 0, score: '-', startedAt, task: 'chess-best-move', tokens: '-', verifierEvidence: '',
        },
      ]

      render(
        <DetailRail
          job={{ ...job, status: 'running' }}
          events={[]}
          trials={trials}
          t={getTranslator('en')}
          onJobAction={vi.fn()}
          onCopyJob={vi.fn()}
          onLeaderboardChange={vi.fn()}
        />,
      )

      const row = screen.getByRole('button', { name: /chess-best-move/ })
      expect(row).toHaveTextContent('00:01:05')
      act(() => { vi.advanceTimersByTime(2_000) })
      expect(row).toHaveTextContent('00:01:07')
    } finally {
      vi.useRealTimers()
    }
  })

  it('renders the raw job log with its path', () => {
    const job = jobs[0]

    render(
      <DetailRail
        job={job}
        events={[]}
        logs={'Running command: claude ...\nagent output line'}
        logsPath={'/tmp/job.log'}
        trials={[]}
        t={getTranslator('en')}
        onJobAction={vi.fn()}
        onCopyJob={vi.fn()}
        onLeaderboardChange={vi.fn()}
      />,
    )

    const log = screen.getByLabelText('Job log')
    expect(log).toHaveTextContent('Running command: claude ...')
    expect(log).toHaveTextContent('agent output line')
    expect(screen.getByText('/tmp/job.log')).toBeInTheDocument()
  })
})

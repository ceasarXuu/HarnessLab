import { render, screen } from '@testing-library/react'
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
        retries: 0, score: '-', task: 'build-cython-ext', tokens: '-', verifierEvidence: '',
      },
      {
        analysisPath: '', artifactPath: '', cost: '$0.50', duration: '00:05:00', id: 'passed-trial',
        jobId: running.id, logPath: '/tmp/other.log', progress: 'passed', result: 'passed',
        retries: 0, score: '1.0/1.0', task: 'sqlite-with-gcov', tokens: '0.1M', verifierEvidence: '',
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
})

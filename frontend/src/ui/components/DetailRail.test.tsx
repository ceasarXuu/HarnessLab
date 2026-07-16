import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
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
})

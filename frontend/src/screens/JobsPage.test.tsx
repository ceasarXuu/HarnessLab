import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
import { getTranslator } from '../i18n'
import { jobs } from '../mocks/demo'
import { JobsPage } from './JobsPage'

describe('JobsPage', () => {
  it('uses the same localized Job status in the list and detail drawer', async () => {
    const job = { ...jobs[0], status: 'running' as const }

    render(
      <JobsPage
        client={createMockWebUiClient()}
        jobs={[job]}
        open
        search=""
        selected={job}
        t={getTranslator('zh')}
        onClose={() => undefined}
        onJobAction={() => undefined}
        onLeaderboardChange={() => undefined}
        onNewJob={() => undefined}
        onSearch={() => undefined}
        onSelect={() => undefined}
      />,
    )

    expect(screen.getAllByText('运行中')).toHaveLength(2)
    expect(screen.queryByText('running')).not.toBeInTheDocument()
  })

  it('requires confirmation before sending a Job cancellation request', async () => {
    const user = userEvent.setup()
    const onJobAction = vi.fn()
    const job = jobs[0]

    render(
      <JobsPage
        client={createMockWebUiClient()}
        jobs={[job]}
        open
        search=""
        selected={job}
        t={getTranslator('en')}
        onClose={() => undefined}
        onJobAction={onJobAction}
        onLeaderboardChange={() => undefined}
        onNewJob={() => undefined}
        onSearch={() => undefined}
        onSelect={() => undefined}
      />,
    )

    await user.click(screen.getByRole('button', { name: 'Cancel' }))

    expect(screen.getByRole('dialog', { name: 'Cancel Job' })).toBeVisible()
    expect(onJobAction).not.toHaveBeenCalled()

    await user.click(screen.getByRole('button', { name: 'Confirm cancel' }))

    expect(onJobAction).toHaveBeenCalledWith(job.id, 'cancel')
  })
})

import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
import { getTranslator } from '../i18n'
import { jobs } from '../mocks/demo'
import { JobsPage } from './JobsPage'

describe('JobsPage', () => {
  it('keeps status in the detail drawer while removing the list status column', async () => {
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
        onCopyJob={() => undefined}
        onLeaderboardChange={() => undefined}
        onNewJob={() => undefined}
        onSearch={() => undefined}
        onSelect={() => undefined}
      />,
    )

    expect(screen.getAllByText('运行中')).toHaveLength(1)
    expect(screen.queryByRole('columnheader', { name: '状态' })).not.toBeInTheDocument()
    expect(screen.queryByText('running')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '删除' })).not.toBeInTheDocument()
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
        onCopyJob={() => undefined}
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

  it('requires explicit confirmation before permanently deleting a terminal Job', async () => {
    const user = userEvent.setup()
    const client = createMockWebUiClient()
    const deleteJob = vi.spyOn(client, 'deleteJob')
    const onRefresh = vi.fn(async () => undefined)
    const job = jobs[1]

    render(
      <JobsPage
        client={client}
        jobs={[job]}
        open
        search=""
        selected={job}
        t={getTranslator('en')}
        onClose={() => undefined}
        onJobAction={() => undefined}
        onCopyJob={() => undefined}
        onLeaderboardChange={() => undefined}
        onNewJob={() => undefined}
        onRefresh={onRefresh}
        onSearch={() => undefined}
        onSelect={() => undefined}
      />,
    )

    await user.click(screen.getByRole('button', { name: 'Delete' }))

    const dialog = screen.getByRole('dialog', { name: 'Delete Job' })
    expect(dialog).toHaveTextContent('Permanently deletes this Job’s Harbor outputs')
    expect(deleteJob).not.toHaveBeenCalled()

    await user.click(screen.getByRole('button', { name: 'Delete permanently' }))

    expect(deleteJob).toHaveBeenCalledWith(job.id)
    expect(onRefresh).toHaveBeenCalled()
  })
})

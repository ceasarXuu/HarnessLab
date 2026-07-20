import { render, screen, within } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { getTranslator } from '../../i18n'
import { jobs } from '../../mocks/demo'
import { JobsTable } from './JobsTable'

describe('JobsTable', () => {
  it('shows task outcome counters without a Job status column', () => {
    render(
      <JobsTable
        jobs={[jobs[0]]}
        search=""
        t={getTranslator('zh')}
        onNewJob={() => undefined}
        onSearch={() => undefined}
        onSelect={() => undefined}
      />,
    )

    expect(screen.queryByRole('columnheader', { name: '状态' })).not.toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: '任务总数' })).toBeVisible()
    expect(screen.getByRole('columnheader', { name: '已完成' })).toBeVisible()
    expect(screen.getByRole('columnheader', { name: '异常失败' })).toBeVisible()

    const row = screen.getByRole('button', { name: 'terminal-bench-smoke' }).closest('tr')
    if (!row) throw new Error('Expected Job row')
    expect(within(row).getByText('64')).toBeVisible()
    expect(within(row).getByText('通过 12')).toBeVisible()
    expect(within(row).getByText('未通过 6')).toBeVisible()
    expect(within(row).getByText('0')).toBeVisible()
  })

  it('adds an accessible loading indicator before a running Job name', () => {
    render(
      <JobsTable
        jobs={[jobs[0]]}
        search=""
        t={getTranslator('zh')}
        onNewJob={() => undefined}
        onSearch={() => undefined}
        onSelect={() => undefined}
      />,
    )

    expect(screen.getByLabelText('运行中')).toHaveClass('job-running-spinner')
  })
})

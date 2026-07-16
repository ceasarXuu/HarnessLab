import { render, screen, within } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
import { systemRows } from '../mocks/demoSystem'
import { getTranslator } from '../i18n'
import { SystemPage } from './SystemPage'

describe('SystemPage', () => {
  it('renders grouped component-specific health cards instead of a generic table', () => {
    render(
      <SystemPage
        client={createMockWebUiClient()}
        rows={systemRows}
        t={getTranslator('en')}
        onRefresh={async () => undefined}
      />,
    )

    expect(screen.queryByRole('table')).not.toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Services & dependencies' })).toBeVisible()
    expect(screen.getByRole('heading', { name: 'Storage' })).toBeVisible()
    expect(screen.getByRole('heading', { name: 'Host resources' })).toBeVisible()

    const service = screen.getByRole('article', { name: 'OrnnLab Service' })
    expect(within(service).getByText('Running')).toBeVisible()
    expect(within(service).getByText('http://127.0.0.1:5173')).toBeVisible()
    expect(within(service).getByText('~/.ornnlab/dev-service/logs')).toBeVisible()
    expect(within(service).getByRole('button', { name: 'Check update' })).toBeVisible()

    const cpu = screen.getByRole('article', { name: 'CPU usage' })
    expect(within(cpu).getByText('12%')).toBeVisible()
    expect(within(cpu).getByText('Normal')).toBeVisible()

    const gpu = screen.getByRole('article', { name: 'GPU usage' })
    expect(within(gpu).getByText('Not detected')).toBeVisible()

    const docker = screen.getByRole('article', { name: 'Docker' })
    expect(within(docker).getByText('Client version')).toBeVisible()
    expect(within(docker).getByText('28.1.1')).toBeVisible()
    expect(within(docker).getByText('Server version')).toBeVisible()
    expect(within(docker).getByText('27.5.1')).toBeVisible()
  })

  it('does not offer Docker cache cleanup while the daemon is stopped', () => {
    const rows = systemRows.map((row) => row.kind === 'docker'
      ? { ...row, state: 'not-running' as const, serverVersion: null, actions: [], error: 'daemon unavailable' }
      : row)

    render(
      <SystemPage
        client={createMockWebUiClient()}
        rows={rows}
        t={getTranslator('en')}
        onRefresh={async () => undefined}
      />,
    )

    const docker = screen.getByRole('article', { name: 'Docker' })
    expect(within(docker).getByText('Not running')).toBeVisible()
    expect(within(docker).getByText('Docker service is not running. Start your local Docker service to use Harbor.')).toBeVisible()
    expect(within(docker).queryByText('daemon unavailable')).not.toBeInTheDocument()
    expect(within(docker).getByText('28.1.1')).toBeVisible()
    expect(within(docker).getByText('—')).toBeVisible()
    expect(within(docker).queryByRole('button', { name: 'Clean cache' })).not.toBeInTheDocument()
  })
})

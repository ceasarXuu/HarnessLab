import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { defaultRunDraft } from '../../domain/defaults'
import { getTranslator } from '../../i18n'
import { RunBuilder } from './RunBuilder'

describe('RunBuilder validation', () => {
  it('explains invalid required fields instead of silently disabling Run job', () => {
    const onLaunch = vi.fn()
    render(
      <RunBuilder
        agents={[]}
        canLaunch
        datasets={[]}
        draft={{ ...defaultRunDraft, jobName: '', jobsDir: '', source: '', agent: '', model: '', environment: '' }}
        environments={[]}
        taskRows={[]}
        t={getTranslator('en')}
        onCancel={vi.fn()}
        onChooseDirectory={vi.fn(async () => ({ path: null }))}
        onCopyJobConfig={vi.fn()}
        onDraft={vi.fn()}
        onLaunch={onLaunch}
        onReset={vi.fn()}
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: 'Run job' }))

    expect(onLaunch).not.toHaveBeenCalled()
    expect(screen.queryByText('Check required fields')).not.toBeInTheDocument()
    expect(screen.getByText('Enter a Job name.')).toBeInTheDocument()
  })
})

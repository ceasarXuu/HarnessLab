import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { FormValidationSummary } from './FormValidationSummary'

describe('FormValidationSummary', () => {
  it('announces issues and lets the user jump to a field', () => {
    const onIssue = vi.fn()
    render(
      <FormValidationSummary
        issues={[{ field: 'name', message: 'Enter a name.' }]}
        title="Check required fields"
        onIssue={onIssue}
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: 'Enter a name.' }))

    expect(screen.getByRole('alert', { name: 'Check required fields' })).toBeInTheDocument()
    expect(onIssue).toHaveBeenCalledWith('name')
  })
})

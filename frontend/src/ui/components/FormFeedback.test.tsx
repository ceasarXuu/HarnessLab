import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { FieldError, FormSubmissionError } from './FormFeedback'

describe('form feedback', () => {
  it('shows field and submission errors without a validation summary', () => {
    render(
      <>
        <FieldError id="name-error" message="Enter a name." />
        <FormSubmissionError message="The request failed." />
      </>,
    )

    expect(screen.getByText('Enter a name.')).toHaveClass('field-error')
    expect(screen.getByRole('alert')).toHaveTextContent('The request failed.')
    expect(screen.queryByText('Check required fields')).not.toBeInTheDocument()
  })
})

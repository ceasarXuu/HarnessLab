import type { Meta, StoryObj } from '@storybook/react-vite'
import { FieldError, FormSubmissionError } from './FormFeedback'

function FormFeedbackStory({ fieldError, submissionError }: { fieldError?: string; submissionError?: string }) {
  return (
    <div style={{ display: 'grid', gap: 16, maxWidth: 480 }}>
      <label>
        Agent Name
        <input aria-describedby={fieldError ? 'storybook-field-error' : undefined} aria-invalid={Boolean(fieldError) || undefined} />
        <FieldError id="storybook-field-error" message={fieldError} />
      </label>
      <FormSubmissionError message={submissionError} />
    </div>
  )
}

const meta = {
  component: FormFeedbackStory,
  title: 'Components/Forms/Feedback',
  args: {
    fieldError: 'Enter a name.',
  },
} satisfies Meta<typeof FormFeedbackStory>

export default meta
type Story = StoryObj<typeof meta>

export const FieldErrors: Story = {
}

export const ServerError: Story = {
  args: {
    fieldError: undefined,
    submissionError: 'The Agent name is already in use.',
  },
}

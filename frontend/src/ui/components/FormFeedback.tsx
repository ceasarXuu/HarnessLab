export interface FormIssue {
  field: string
  message: string
}

export function FormSubmissionError({ message }: { message?: string | null }) {
  if (!message) return null
  return <p className="form-submission-error" role="alert">{message}</p>
}

export function FieldError({ id, message }: { id: string; message?: string }) {
  if (!message) return null
  return <span className="field-error" id={id}>{message}</span>
}

export function issuesByField(issues: FormIssue[]) {
  return Object.fromEntries(issues.map((issue) => [issue.field, issue.message]))
}

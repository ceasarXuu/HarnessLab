export interface FormIssue {
  field: string
  message: string
}

interface FormValidationSummaryProps {
  issues: FormIssue[]
  title: string
  serverError?: string | null
  onIssue?: (field: string) => void
}

export function FormValidationSummary({ issues, onIssue, serverError, title }: FormValidationSummaryProps) {
  if (!issues.length && !serverError) return null
  return (
    <section className="form-validation-summary" role="alert" aria-label={title} tabIndex={-1}>
      <strong>{title}</strong>
      {serverError && <p>{serverError}</p>}
      {issues.length > 0 && (
        <ul>
          {issues.map((issue) => (
            <li key={issue.field}>
              <button type="button" onClick={() => onIssue?.(issue.field)}>{issue.message}</button>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}

export function FieldError({ id, message }: { id: string; message?: string }) {
  if (!message) return null
  return <span className="field-error" id={id}>{message}</span>
}

export function issuesByField(issues: FormIssue[]) {
  return Object.fromEntries(issues.map((issue) => [issue.field, issue.message]))
}

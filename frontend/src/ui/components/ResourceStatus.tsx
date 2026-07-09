interface ResourceStatusProps {
  error: string | null
  loading: boolean
  loadingLabel: string
}

export function ResourceStatus({ error, loading, loadingLabel }: ResourceStatusProps) {
  if (error) return <div className="resource-state error" role="alert">{error}</div>
  if (loading) return <div className="resource-state" role="status">{loadingLabel}</div>
  return null
}

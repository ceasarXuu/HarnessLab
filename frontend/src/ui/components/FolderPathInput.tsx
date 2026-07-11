import { FolderOpen } from 'lucide-react'
import { useState } from 'react'

export interface FolderPathSelection {
  error?: string
  path: string | null
}

interface FolderPathInputProps {
  chooseLabel: string
  label: string
  onChoose: () => Promise<FolderPathSelection>
  value: string
  onChange: (value: string) => void
}

export function FolderPathInput({ chooseLabel, label, onChange, onChoose, value }: FolderPathInputProps) {
  const [choosing, setChoosing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const choose = async () => {
    setChoosing(true)
    setError(null)
    try {
      const result = await onChoose()
      if (result.path) onChange(result.path)
      if (result.error) setError(result.error)
    } finally {
      setChoosing(false)
    }
  }

  return (
    <div className="folder-path-input">
      <input aria-label={label} readOnly value={value} />
      <button className="secondary-button compact-action" disabled={choosing} type="button" onClick={() => void choose()}>
        <FolderOpen aria-hidden="true" />
        {chooseLabel}
      </button>
      {error && <p className="folder-path-picker-error" role="alert">{error}</p>}
    </div>
  )
}

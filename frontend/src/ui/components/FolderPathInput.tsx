import { FolderOpen } from 'lucide-react'
import { useRef } from 'react'

interface FolderPathInputProps {
  chooseLabel: string
  label: string
  value: string
  onChange: (value: string) => void
}

export function FolderPathInput({ chooseLabel, label, onChange, value }: FolderPathInputProps) {
  const fileInputRef = useRef<HTMLInputElement>(null)

  const updateFromFiles = (files: FileList | null) => {
    const firstFile = files?.item(0)
    const relativePath = firstFile?.webkitRelativePath
    const folderName = relativePath?.split('/').filter(Boolean)[0]
    if (folderName) {
      onChange(folderName)
    }
  }

  return (
    <div className="folder-path-input">
      <input value={value} onChange={(event) => onChange(event.target.value)} />
      <button className="secondary-button compact-action" type="button" onClick={() => fileInputRef.current?.click()}>
        <FolderOpen aria-hidden="true" />
        {chooseLabel}
      </button>
      <input
        ref={fileInputRef}
        aria-label={label}
        className="visually-hidden"
        type="file"
        onChange={(event) => updateFromFiles(event.target.files)}
        {...{ directory: '', webkitdirectory: '' }}
      />
    </div>
  )
}

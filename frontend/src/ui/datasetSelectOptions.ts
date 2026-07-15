import type { DatasetRow } from '../domain/harbor'
import type { Translate } from '../i18n'
import type { SelectOption } from './components/CustomSelect'

export function datasetRef(row: DatasetRow): string {
  return row.ref ?? `${row.name}@${row.version}`
}

export function datasetSelectOptions(rows: DatasetRow[], t: Translate): SelectOption[] {
  return rows.map((row) => ({
    badge: row.downloadStatus === 'downloaded'
      ? { label: t('downloaded'), tone: 'success' }
      : row.downloadStatus === 'path-unavailable'
        ? { label: t('pathUnavailable'), tone: 'warning' }
        : { label: t('notDownloaded'), tone: 'neutral' },
    label: datasetRef(row),
    value: datasetRef(row),
  }))
}

import type { DatasetRow } from './harbor'

export function orderDatasetCatalog(rows: DatasetRow[]): DatasetRow[] {
  return [...rows].sort((left, right) => {
    const leftDownloaded = left.downloadStatus === 'downloaded'
    const rightDownloaded = right.downloadStatus === 'downloaded'
    if (leftDownloaded !== rightDownloaded) return leftDownloaded ? -1 : 1
    if (leftDownloaded && rightDownloaded) {
      return timestamp(right.downloadedAt) - timestamp(left.downloadedAt)
    }
    return left.name.localeCompare(right.name) || left.version.localeCompare(right.version)
  })
}

function timestamp(value: string | undefined): number {
  if (!value) return 0
  const parsed = Date.parse(value)
  return Number.isFinite(parsed) ? parsed : 0
}

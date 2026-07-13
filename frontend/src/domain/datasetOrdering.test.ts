import { describe, expect, it } from 'vitest'
import type { DatasetRow } from './harbor'
import { orderDatasetCatalog } from './datasetOrdering'

describe('orderDatasetCatalog', () => {
  it('places downloaded Datasets first by recent download time and sorts the rest by name', () => {
    const rows: DatasetRow[] = [
      dataset('zeta', 'not-downloaded'),
      dataset('older', 'downloaded', '2026-07-10T00:00:00Z'),
      dataset('alpha', 'not-downloaded'),
      dataset('newer', 'downloaded', '2026-07-12T00:00:00Z'),
    ]

    expect(orderDatasetCatalog(rows).map((row) => row.name)).toEqual(['newer', 'older', 'alpha', 'zeta'])
  })
})

function dataset(name: string, downloadStatus: DatasetRow['downloadStatus'], downloadedAt?: string): DatasetRow {
  return {
    downloadStatus,
    downloadedAt,
    name,
    source: 'harbor registry',
    tasks: 1,
    version: '1.0',
    visibility: 'public',
  }
}

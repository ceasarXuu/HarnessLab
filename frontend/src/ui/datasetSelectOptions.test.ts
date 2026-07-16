import { describe, expect, it } from 'vitest'
import type { DatasetRow } from '../domain/harbor'
import { getTranslator } from '../i18n'
import { datasetSelectOptions } from './datasetSelectOptions'

describe('datasetSelectOptions', () => {
  it('places downloaded Datasets before unavailable catalog entries', () => {
    const rows = [
      dataset('alpha', 'not-downloaded'),
      dataset('older', 'downloaded', '2026-07-10T00:00:00Z'),
      dataset('newer', 'downloaded', '2026-07-12T00:00:00Z'),
      dataset('missing', 'path-unavailable'),
    ]

    expect(datasetSelectOptions(rows, getTranslator('en')).map((option) => option.value)).toEqual([
      'newer@1.0',
      'older@1.0',
      'alpha@1.0',
      'missing@1.0',
    ])
  })
})

function dataset(
  name: string,
  downloadStatus: DatasetRow['downloadStatus'],
  downloadedAt?: string,
): DatasetRow {
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

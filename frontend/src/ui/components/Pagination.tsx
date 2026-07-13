import { ChevronLeft, ChevronRight } from 'lucide-react'
import type { Translate } from '../../i18n'

interface PaginationProps {
  endItem: number
  page: number
  startItem: number
  t: Translate
  totalItems: number
  totalPages: number
  onPage: (page: number) => void
}

export function Pagination({
  endItem,
  page,
  startItem,
  t,
  totalItems,
  totalPages,
  onPage,
}: PaginationProps) {
  if (totalItems === 0) return null
  return (
    <nav aria-label={t('pagination')} className="pagination-bar">
      <span>
        {t('paginationRange')
          .replace('{start}', String(startItem))
          .replace('{end}', String(endItem))
          .replace('{total}', String(totalItems))}
      </span>
      <div className="pagination-controls">
        <button
          aria-label={t('previousPage')}
          className="icon-button"
          disabled={page <= 1}
          type="button"
          onClick={() => onPage(page - 1)}
        >
          <ChevronLeft aria-hidden="true" />
        </button>
        <span>{t('paginationPage').replace('{page}', String(page)).replace('{totalPages}', String(totalPages))}</span>
        <button
          aria-label={t('nextPage')}
          className="icon-button"
          disabled={page >= totalPages}
          type="button"
          onClick={() => onPage(page + 1)}
        >
          <ChevronRight aria-hidden="true" />
        </button>
      </div>
    </nav>
  )
}

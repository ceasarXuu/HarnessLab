import { fireEvent, render, screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { CustomSelect } from './CustomSelect'

function options(count: number) {
  return Array.from({ length: count }, (_, index) => ({
    label: `Option ${index + 1}`,
    value: String(index + 1),
  }))
}

describe('CustomSelect', () => {
  it('keeps lists with ten options compact', () => {
    render(<CustomSelect ariaLabel="Compact" options={options(10)} value="" onChange={() => undefined} />)
    fireEvent.click(screen.getByLabelText('Compact'))

    expect(screen.queryByRole('textbox', { name: 'Search Compact' })).not.toBeInTheDocument()
    expect(screen.getAllByRole('option')).toHaveLength(10)
  })

  it('automatically searches lists with more than ten options', () => {
    const onChange = vi.fn()
    render(<CustomSelect ariaLabel="Large" options={options(11)} value="" onChange={onChange} />)
    fireEvent.click(screen.getByLabelText('Large'))

    const search = screen.getByRole('textbox', { name: 'Search Large' })
    fireEvent.change(search, { target: { value: 'Option 11' } })
    const listbox = screen.getByRole('listbox', { name: 'Large options' })
    expect(within(listbox).getAllByRole('option')).toHaveLength(1)
    fireEvent.click(within(listbox).getByRole('option', { name: 'Option 11' }))
    expect(onChange).toHaveBeenCalledWith('11')

    fireEvent.click(screen.getByLabelText('Large'))
    expect(screen.getByRole('textbox', { name: 'Search Large' })).toHaveValue('')
    expect(screen.getAllByRole('option')).toHaveLength(11)
  })
})

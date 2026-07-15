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
  it('exposes asynchronous option loading through the control busy state', () => {
    render(<CustomSelect ariaLabel="Async" busy options={options(2)} value="" onChange={() => undefined} />)

    expect(screen.getByLabelText('Async')).toHaveAttribute('aria-busy', 'true')
  })

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

  it('renders option badges without including them in local search matching', () => {
    render(
      <CustomSelect
        ariaLabel="Dataset"
        options={[
          { badge: { label: 'Downloaded', tone: 'success' }, label: 'terminal-bench@2.0', value: 'terminal-bench@2.0' },
          { badge: { label: 'Not downloaded', tone: 'neutral' }, label: 'swebench@1.0', value: 'swebench@1.0' },
        ]}
        searchable
        value="terminal-bench@2.0"
        onChange={() => undefined}
      />,
    )

    expect(screen.getByLabelText('Dataset')).toHaveTextContent('terminal-bench@2.0Downloaded')
    fireEvent.click(screen.getByLabelText('Dataset'))
    expect(screen.getByRole('option', { name: 'terminal-bench@2.0 Downloaded' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: 'swebench@1.0 Not downloaded' })).toBeInTheDocument()

    fireEvent.change(screen.getByRole('textbox', { name: 'Search Dataset' }), { target: { value: 'Downloaded' } })
    expect(screen.queryAllByRole('option')).toHaveLength(0)
  })
})

import { act, fireEvent, render, screen } from '@testing-library/react'
import { useState } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { useDebouncedAutosave } from './useDebouncedAutosave'

function AutosaveFixture({ onSave }: { onSave: (value: string) => boolean | Promise<boolean> }) {
  const [value, setValue] = useState('initial')
  useDebouncedAutosave({ value, onSave })
  return <input aria-label="Value" value={value} onChange={(event) => setValue(event.target.value)} />
}

describe('useDebouncedAutosave', () => {
  afterEach(() => vi.useRealTimers())

  it('saves only the latest value after the debounce window', async () => {
    vi.useFakeTimers()
    const onSave = vi.fn(async () => true)
    render(<AutosaveFixture onSave={onSave} />)

    fireEvent.change(screen.getByLabelText('Value'), { target: { value: 'first' } })
    fireEvent.change(screen.getByLabelText('Value'), { target: { value: 'latest' } })
    await act(() => vi.advanceTimersByTimeAsync(400))

    expect(onSave).toHaveBeenCalledTimes(1)
    expect(onSave).toHaveBeenCalledWith('latest')
  })

  it('queues the latest edit while a save is in flight', async () => {
    vi.useFakeTimers()
    let completeFirstSave: ((value: boolean) => void) | undefined
    const onSave = vi.fn(() => new Promise<boolean>((resolve) => { completeFirstSave ??= resolve }))
    render(<AutosaveFixture onSave={onSave} />)

    fireEvent.change(screen.getByLabelText('Value'), { target: { value: 'first' } })
    await act(() => vi.advanceTimersByTimeAsync(400))
    fireEvent.change(screen.getByLabelText('Value'), { target: { value: 'latest' } })
    await act(() => vi.advanceTimersByTimeAsync(400))
    expect(onSave).toHaveBeenCalledTimes(1)

    await act(async () => completeFirstSave?.(true))
    expect(onSave).toHaveBeenCalledTimes(2)
    expect(onSave).toHaveBeenLastCalledWith('latest')
  })
})

import { fireEvent, render, screen } from '@testing-library/react'
import { useState } from 'react'
import { describe, expect, it } from 'vitest'
import { EditableStringList } from './EditableStringList'
import { KeyValueControl } from './KeyValueControl'

describe('editable list controls', () => {
  it('removes a newly added empty string row when it loses focus', () => {
    render(<StringListFixture />)

    fireEvent.click(screen.getByRole('button', { name: 'Add' }))
    expect(screen.getByRole('textbox', { name: 'Models 1' })).toBeInTheDocument()

    fireEvent.blur(screen.getByRole('textbox', { name: 'Models 1' }), {
      relatedTarget: screen.getByRole('button', { name: 'Outside' }),
    })

    expect(screen.queryByRole('textbox', { name: 'Models 1' })).not.toBeInTheDocument()
  })

  it('keeps a blank key-value row while focus moves inside it and removes it on exit', () => {
    render(<KeyValueFixture />)

    fireEvent.click(screen.getByRole('button', { name: 'Add' }))
    const keyInput = screen.getByRole('textbox', { name: 'Key' })
    const valueInput = screen.getByRole('textbox', { name: 'Value' })

    fireEvent.blur(keyInput, { relatedTarget: valueInput })
    expect(valueInput).toBeInTheDocument()
    fireEvent.blur(valueInput, { relatedTarget: screen.getByRole('button', { name: 'Outside' }) })
    expect(screen.queryByRole('textbox', { name: 'Key' })).not.toBeInTheDocument()
  })

  it('keeps a string row after the user enters content', () => {
    render(<StringListFixture />)

    fireEvent.click(screen.getByRole('button', { name: 'Add' }))
    fireEvent.change(screen.getByRole('textbox', { name: 'Models 1' }), { target: { value: 'claude-sonnet-4-5' } })
    fireEvent.blur(screen.getByRole('textbox', { name: 'Models 1' }), {
      relatedTarget: screen.getByRole('button', { name: 'Outside' }),
    })

    expect(screen.getByRole('textbox', { name: 'Models 1' })).toHaveValue('claude-sonnet-4-5')
  })

  it('serializes inherited and fixed environment variable values distinctly', () => {
    render(<InheritedKeyValueFixture />)

    expect(screen.getByRole('combobox', { name: 'Value source' })).toHaveValue('inherited')
    expect(screen.getByRole('textbox', { name: 'Value' })).toBeDisabled()
    fireEvent.change(screen.getByRole('combobox', { name: 'Value source' }), { target: { value: 'literal' } })
    fireEvent.change(screen.getByRole('textbox', { name: 'Value' }), { target: { value: 'secret-reference' } })

    expect(screen.getByTestId('serialized-env')).toHaveTextContent('API_KEY=secret-reference')
  })
})

function StringListFixture() {
  const [values, setValues] = useState<string[]>([])
  return (
    <>
      <EditableStringList addLabel="Add" deleteLabel="Delete" label="Models" values={values} onChange={setValues} />
      <button type="button">Outside</button>
    </>
  )
}

function KeyValueFixture() {
  const [value, setValue] = useState('none')
  return (
    <>
      <KeyValueControl
        label="Environment"
        labels={{ add: 'Add', delete: 'Delete', key: 'Key', value: 'Value' }}
        value={value}
        onChange={setValue}
      />
      <button type="button">Outside</button>
    </>
  )
}

function InheritedKeyValueFixture() {
  const [value, setValue] = useState('API_KEY')
  return (
    <>
      <KeyValueControl
        allowInherited
        compact
        label="Environment"
        labels={{
          add: 'Add', delete: 'Delete', inherited: 'Inherit system variable', key: 'Key',
          literal: 'Fixed value', source: 'Value source', value: 'Value',
        }}
        value={value}
        onChange={setValue}
      />
      <output data-testid="serialized-env">{value}</output>
    </>
  )
}

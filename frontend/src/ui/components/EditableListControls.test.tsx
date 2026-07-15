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

  it('hides the value input for inherited variables and restores it for custom values', () => {
    render(<InheritedKeyValueFixture />)

    expect(screen.getByRole('button', { name: 'Value source' })).toHaveTextContent('Inherit system variable')
    expect(screen.queryByRole('textbox', { name: 'Value' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Value source' }))
    fireEvent.click(screen.getByRole('option', { name: 'Custom' }))
    expect(screen.getByRole('textbox', { name: 'Value' })).toBeEnabled()
    fireEvent.change(screen.getByRole('textbox', { name: 'Value' }), { target: { value: 'secret-reference' } })

    expect(screen.getByTestId('serialized-env')).toHaveTextContent('API_KEY=secret-reference')
  })

  it('offers known Harbor variables and keeps a custom variable escape hatch', () => {
    render(<KnownEnvironmentFixture />)

    fireEvent.click(screen.getByRole('button', { name: 'Add variable Environment' }))
    expect(screen.getByRole('button', { name: 'Value source' })).toHaveTextContent('Custom')
    expect(screen.getByRole('textbox', { name: 'Value' })).toBeEnabled()
    expect(screen.queryByRole('combobox', { name: 'Value source' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Variable name' }))
    fireEvent.click(screen.getByRole('option', { name: 'OPENAI_API_KEY' }))
    expect(screen.getByTestId('serialized-known-env')).toHaveTextContent('OPENAI_API_KEY')

    fireEvent.click(screen.getByRole('button', { name: 'Add variable Environment' }))
    fireEvent.click(screen.getAllByRole('button', { name: 'Variable name' })[1])
    fireEvent.click(screen.getByRole('option', { name: 'Custom variable' }))
    fireEvent.change(screen.getByRole('textbox', { name: 'Variable name' }), {
      target: { value: 'MY_CUSTOM_TOKEN' },
    })
    expect(screen.getByTestId('serialized-known-env')).toHaveTextContent('MY_CUSTOM_TOKEN')
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
          literal: 'Custom', source: 'Value source', value: 'Value',
        }}
        value={value}
        onChange={setValue}
      />
      <output data-testid="serialized-env">{value}</output>
    </>
  )
}

function KnownEnvironmentFixture() {
  const [value, setValue] = useState('none')
  return (
    <>
      <KeyValueControl
        allowInherited
        compact
        keyOptions={['OPENAI_API_KEY', 'OPENAI_BASE_URL']}
        label="Environment"
        labels={{
          add: 'Add variable', customKey: 'Custom variable', delete: 'Delete',
          inherited: 'Inherit system variable', key: 'Variable name',
          literal: 'Custom', searchKeys: 'Search variables', source: 'Value source',
          value: 'Value',
        }}
        value={value}
        onChange={setValue}
      />
      <output data-testid="serialized-known-env">{value}</output>
    </>
  )
}

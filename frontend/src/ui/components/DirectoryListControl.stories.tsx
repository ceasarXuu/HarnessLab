import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, within } from 'storybook/test'
import { useState } from 'react'
import { DirectoryListControl } from './AgentProfileEditor'

function SkillsDirectoryFixture() {
  const [value, setValue] = useState('~/.ornnlab/skills')
  return (
    <section className="surface rail-card">
      <DirectoryListControl
        addLabel="Add"
        chooseLabel="Choose folder"
        deleteLabel="Delete"
        description="Choose a skill directory or a folder containing multiple skill directories."
        label="Skills"
        value={value}
        onChange={setValue}
      />
    </section>
  )
}

const meta = {
  title: 'Patterns/Agent/SkillsDirectory',
  parameters: { layout: 'padded' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof SkillsDirectoryFixture>

export const FolderSelectionOnly: Story = {
  render: () => <SkillsDirectoryFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByRole('button', { name: 'Choose folder' })).toBeVisible()
    await expect(canvas.queryByRole('button', { name: 'Add' })).not.toBeInTheDocument()
    await expect(canvas.getByLabelText('Skills')).toHaveValue('~/.ornnlab/skills')
  },
}

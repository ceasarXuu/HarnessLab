import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { McpServersControl, type McpServerLabels } from './McpServersControl'

const labels: McpServerLabels = {
  addItem: 'Add',
  addServer: 'Add MCP Server',
  args: 'Args',
  command: 'Command',
  description: 'Configure MCP sidecar servers for this harness. HTTP transports use URL; stdio uses command, args, and env.',
  enabled: 'Enabled',
  env: 'Env',
  name: 'Name',
  transport: 'Transport',
  url: 'URL',
}

function McpServersFixture() {
  const [value, setValue] = useState(JSON.stringify([
    {
      enabled: true,
      name: 'terminal-bench-mcp',
      transport: 'streamable-http',
      url: 'http://mcp-server:8000/mcp',
    },
  ]))

  return (
    <main className="workspace single-page">
      <section className="surface rail-card">
        <McpServersControl labels={labels} value={value} onChange={setValue} />
      </section>
    </main>
  )
}

const meta = {
  title: 'Components/MCP Servers Control',
  component: McpServersFixture,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof McpServersFixture>

export default meta
type Story = StoryObj<typeof meta>

export const StreamableHttp: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByLabelText('Name')).toHaveValue('terminal-bench-mcp')
    await expect(canvas.getByLabelText('Transport')).toHaveValue('streamable-http')
    await expect(canvas.getByLabelText('URL')).toHaveValue('http://mcp-server:8000/mcp')
  },
}

export const AddStdioServer: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByRole('button', { name: 'Add MCP Server' }))
    const transports = canvas.getAllByLabelText('Transport')
    await userEvent.selectOptions(transports[1], 'stdio')
    await expect(canvas.getByLabelText('Command')).toBeVisible()
    await userEvent.type(canvas.getByLabelText('Command'), 'uvx')
    await userEvent.type(canvas.getByLabelText('Args'), 'repair-tools-mcp')
    await expect(canvas.getByLabelText('Command')).toHaveValue('uvx')
    await expect(canvas.getByLabelText('Args')).toHaveValue('repair-tools-mcp')
  },
}

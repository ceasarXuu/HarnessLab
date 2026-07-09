import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { McpServersControl, type McpServerLabels } from './McpServersControl'

const labels: McpServerLabels = {
  addItem: 'Add',
  addServer: 'Add MCP Server',
  args: 'Args',
  composeSidecar: 'Docker Compose sidecar',
  composeYaml: 'Compose YAML',
  command: 'Command',
  deleteItem: 'Delete',
  deleteServer: 'Delete MCP Server',
  deployment: 'Deployment',
  description: 'Manage MCP templates on the Agent. OrnnLab expands compose sidecars into Harbor task environment and registers the generated connection in task.toml.',
  enabled: 'Enabled',
  endpointPath: 'Endpoint path',
  env: 'Env',
  externalService: 'External service',
  generatedUrl: 'Generated URL',
  key: 'Env key',
  name: 'Name',
  port: 'Port',
  serviceName: 'Service name',
  stdio: 'stdio command',
  transport: 'Transport',
  url: 'URL',
  value: 'Env value',
}

function McpServersFixture() {
  const [value, setValue] = useState(JSON.stringify([
    {
      composeYaml: 'services:\n  terminal-bench-mcp:\n    image: terminal-bench-mcp:latest\n    expose:\n      - "8000"',
      deployment: 'compose-sidecar',
      endpointPath: '/mcp',
      enabled: true,
      name: 'terminal-bench-mcp',
      port: '8000',
      serviceName: 'terminal-bench-mcp',
      transport: 'streamable-http',
      url: 'http://terminal-bench-mcp:8000/mcp',
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
    await expect(canvas.getByLabelText('Deployment')).toHaveValue('compose-sidecar')
    await expect(canvas.getByLabelText('Transport')).toHaveValue('streamable-http')
    await expect(canvas.getByLabelText('Generated URL')).toHaveValue('http://terminal-bench-mcp:8000/mcp')
    await expect(canvas.getByLabelText('Compose YAML')).toHaveValue(expect.stringContaining('services:'))
  },
}

export const AddStdioServer: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByRole('button', { name: 'Add MCP Server' }))
    const deployments = canvas.getAllByLabelText('Deployment')
    await userEvent.selectOptions(deployments[1], 'stdio')
    await expect(canvas.getByLabelText('Command')).toBeVisible()
    await userEvent.type(canvas.getByLabelText('Command'), 'uvx')
    await userEvent.type(canvas.getByLabelText('Args'), 'repair-tools-mcp')
    await expect(canvas.getByLabelText('Command')).toHaveValue('uvx')
    await expect(canvas.getByLabelText('Args')).toHaveValue('repair-tools-mcp')
  },
}

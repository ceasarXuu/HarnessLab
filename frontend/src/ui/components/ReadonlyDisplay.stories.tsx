import type { Meta, StoryObj } from '@storybook/react-vite'
import { ReadonlyKeyValueList, ReadonlyMcpServers, ReadonlyStringList } from './ReadonlyDisplay'

const mcpLabels = {
  addServer: 'Add server',
  args: 'Args',
  command: 'Command',
  deleteItem: 'Delete item',
  deleteServer: 'Delete server',
  name: 'MCP servers',
  transport: 'Transport',
  url: 'URL',
}

const meta = {
  title: 'Components/ReadonlyDisplay',
  parameters: { layout: 'padded' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const KeyValue: Story = {
  render: () => (
    <ReadonlyKeyValueList
      emptyLabel="Configured at job run"
      label="Agent env"
      value={'ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}\nANTHROPIC_BASE_URL=${ANTHROPIC_BASE_URL}'}
    />
  ),
}

export const StringList: Story = {
  render: () => (
    <ReadonlyStringList
      emptyLabel="Supported by harness"
      label="Skills"
      value={'~/.ornnlab/skills/terminal-bench\n~/.ornnlab/skills/swe-bench'}
    />
  ),
}

export const McpServers: Story = {
  render: () => (
    <ReadonlyMcpServers
      emptyLabel="Supported by harness"
      label="MCP config"
      labels={mcpLabels}
      value={JSON.stringify([{ args: ['--workspace', '/workspace'], command: 'npx terminal-mcp', name: 'terminal', transport: 'stdio' }])}
    />
  ),
}

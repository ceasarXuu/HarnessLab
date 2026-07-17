import type { Meta, StoryObj } from '@storybook/react-vite'
import { getTranslator } from '../../i18n'
import { DatasetTaskEnvironment } from './DatasetTaskEnvironment'

const meta = {
  component: DatasetTaskEnvironment,
  parameters: { layout: 'padded' },
  title: 'Dataset/DatasetTaskEnvironment',
} satisfies Meta<typeof DatasetTaskEnvironment>

export default meta
type Story = StoryObj<typeof meta>

export const ParsedEnvironment: Story = {
  args: {
    environment: {
      allowedHosts: ['api.example.com'],
      buildTimeoutSeconds: 900,
      definitions: ['docker-image', 'dockerfile'],
      dockerImage: 'ghcr.io/example/task:2.0',
      networkMode: 'allowlist',
      os: 'linux',
      resources: { cpus: 4, gpuTypes: ['A100'], gpus: 1, memoryMb: 8192, storageMb: 20480, tpu: null },
      workdir: '/workspace',
    },
    t: getTranslator('zh'),
  },
}

export const RegistryMetadataOnly: Story = {
  args: { environment: null, t: getTranslator('zh') },
}

import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, within } from 'storybook/test'
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
      containerImages: [
        { platforms: ['linux/amd64'], reference: 'ghcr.io/example/task:2.0', source: 'environment-config' },
        { platforms: ['linux/amd64', 'linux/arm64/v8'], reference: 'python:3.13-slim', source: 'dockerfile-base' },
      ],
      definitions: ['docker-image', 'dockerfile'],
      networkMode: 'allowlist',
      os: 'linux',
      resources: { cpus: 4, gpuTypes: ['A100'], gpus: 1, memoryMb: 8192, storageMb: 20480, tpu: null },
      workdir: '/workspace',
    },
    t: getTranslator('zh'),
  },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByText('预构建环境镜像')).toBeVisible()
    await expect(canvas.getByText('ghcr.io/example/task:2.0')).toBeVisible()
    await expect(canvas.getByText('Dockerfile 基础镜像')).toBeVisible()
    await expect(canvas.getByText('python:3.13-slim')).toBeVisible()
    await expect(canvas.getByText('linux/amd64, linux/arm64/v8')).toBeVisible()
  },
}

export const RegistryMetadataOnly: Story = {
  args: { environment: null, t: getTranslator('zh') },
}

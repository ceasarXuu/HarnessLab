import type { StorybookConfig } from '@storybook/vue3-vite'

const config: StorybookConfig = {
  framework: {
    name: '@storybook/vue3-vite',
    options: {},
  },
  stories: ['../src/**/*.stories.ts'],
}

export default config


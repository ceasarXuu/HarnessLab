import type { Preview } from '@storybook/react-vite'
import { initialize, mswLoader } from 'msw-storybook-addon'
import { webuiHandlers } from '../src/mocks/mswHandlers'
import '../src/styles/index.css'

initialize({ onUnhandledRequest: 'bypass' })

const preview: Preview = {
  decorators: [
    (Story, context) => {
      const theme = context.globals.theme === 'light' ? 'light' : 'dark'
      const locale = context.globals.locale === 'zh' ? 'zh' : 'en'
      document.documentElement.dataset.theme = theme
      document.documentElement.lang = locale
      window.localStorage.setItem('ornnlab.theme', theme)
      window.localStorage.setItem('ornnlab.locale', locale)
      return Story()
    },
  ],
  globalTypes: {
    locale: {
      defaultValue: 'en',
      description: 'Reader locale',
      toolbar: {
        icon: 'globe',
        items: [
          { title: 'English', value: 'en' },
          { title: '中文', value: 'zh' },
        ],
      },
    },
    theme: {
      defaultValue: 'dark',
      description: 'Color theme',
      toolbar: {
        icon: 'mirror',
        items: [
          { title: 'Dark', value: 'dark' },
          { title: 'Light', value: 'light' },
        ],
      },
    },
  },
  loaders: [mswLoader],
  parameters: {
    a11y: {
      test: 'error',
    },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    msw: {
      handlers: webuiHandlers,
    },
  },
}

export default preview

import type { Preview } from '@storybook/vue3-vite'
import { setup } from '@storybook/vue3-vite'

import { i18n } from '../src/i18n'
import '../src/styles.css'

setup((app) => {
  app.use(i18n)
})

const preview: Preview = {
  parameters: {
    layout: 'fullscreen',
  },
}

export default preview

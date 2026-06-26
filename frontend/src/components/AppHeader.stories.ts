import type { Meta, StoryObj } from '@storybook/vue3-vite'

import AppHeader from './AppHeader.vue'
import { setLocale } from '@/i18n'

const cacheStars = (count: number) => {
  sessionStorage.setItem(
    'ornnlab.stars.ceasarXuu/HarnessLab',
    JSON.stringify({ count, fetchedAt: Date.now() }),
  )
}

const meta = {
  title: 'Console/AppHeader',
  component: AppHeader,
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof AppHeader>

export default meta

type Story = StoryObj<typeof meta>

export const EnglishLoaded: Story = {
  decorators: [
    (story) => {
      setLocale('en')
      document.documentElement.setAttribute('data-theme', 'light')
      cacheStars(1284)
      return story()
    },
  ],
}

export const ChineseDarkLoaded: Story = {
  decorators: [
    (story) => {
      setLocale('zh')
      document.documentElement.setAttribute('data-theme', 'dark')
      cacheStars(1284)
      return story()
    },
  ],
}

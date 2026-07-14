import { describe, expect, it } from 'vitest'
import { devServerWatchIgnored } from '../config/viteWatch'

describe('devServerWatchIgnored', () => {
  it('keeps generated artifacts out of the dev server watcher', () => {
    expect(devServerWatchIgnored).toEqual(
      expect.arrayContaining([
        '**/dist/**',
        '**/coverage/**',
        '**/storybook-static/**',
        '**/node_modules/.vite/**',
      ]),
    )
  })
})

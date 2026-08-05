import { mergeConfig } from 'vite'
import { createWebUiViteConfig } from './vite.config'

export default mergeConfig(createWebUiViteConfig('serve'), {
  test: {
    environment: 'jsdom',
    environmentOptions: {
      jsdom: { url: 'http://localhost:3000' },
    },
    setupFiles: './src/test/setup.ts',
    exclude: ['tests/e2e/**', 'node_modules/**', 'dist/**'],
  },
})

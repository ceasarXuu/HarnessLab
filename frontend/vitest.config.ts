import { mergeConfig } from 'vite'
import { createWebUiViteConfig } from './vite.config'

export default mergeConfig(createWebUiViteConfig('serve'), {
  test: {
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    exclude: ['tests/e2e/**', 'node_modules/**', 'dist/**'],
  },
})

import { mergeConfig } from 'vite'
import baseConfig from './vite.config'

export default mergeConfig(baseConfig, {
  test: {
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    exclude: ['tests/e2e/**', 'node_modules/**', 'dist/**'],
  },
})

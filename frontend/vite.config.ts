import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig, type ConfigEnv, type UserConfig } from 'vite'
import { resolveWebUiDataMode } from './src/api/dataMode'
import { devServerWatchIgnored } from './src/config/viteWatch'

export function createWebUiViteConfig(command: ConfigEnv['command']): UserConfig {
  const dataMode = resolveWebUiDataMode(
    process.env.VITE_ORNNLAB_DATA_MODE,
    command === 'build' ? 'api' : 'mock',
  )
  if (command === 'build' && dataMode !== 'api') {
    throw new Error('Production WebUI builds require VITE_ORNNLAB_DATA_MODE=api.')
  }
  process.env.VITE_ORNNLAB_DATA_MODE = dataMode
  const frontendPort = readPort(process.env.ORNNLAB_FRONTEND_PORT, 5173)

  return {
    plugins: [tailwindcss(), react()],
    server: {
      host: '127.0.0.1',
      port: frontendPort,
      strictPort: true,
      watch: {
        ignored: devServerWatchIgnored,
      },
      proxy: {
        '/api': {
          target: process.env.ORNNLAB_API_TARGET ?? 'http://127.0.0.1:8765',
          changeOrigin: true,
        },
      },
    },
    preview: {
      host: '127.0.0.1',
      port: readPort(process.env.ORNNLAB_PREVIEW_PORT, 4173),
      strictPort: true,
    },
  }
}

export default defineConfig(({ command }) => createWebUiViteConfig(command))

function readPort(value: string | undefined, fallback: number): number {
  if (value === undefined || value === '') return fallback
  const port = Number(value)
  if (!Number.isInteger(port) || port < 1 || port > 65_535) {
    throw new Error(`Expected a TCP port from 1 to 65535, received "${value}".`)
  }
  return port
}

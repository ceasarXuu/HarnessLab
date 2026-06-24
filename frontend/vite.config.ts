import { fileURLToPath, URL } from 'node:url'

import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

// 后端默认监听 127.0.0.1:8765（见 ornnlab/cli.py:24、ornnlab/settings.py:18）。
// 通过 ORNNLAB_API_TARGET 环境变量可覆盖；生产部署形态延后到 v0.1.5 PRD 决定。
const apiTarget = process.env.ORNNLAB_API_TARGET ?? 'http://127.0.0.1:8765'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    host: '127.0.0.1',
    port: 4173,
    proxy: {
      '/api': {
        target: apiTarget,
        changeOrigin: true,
      },
    },
  },
  preview: {
    host: '127.0.0.1',
    port: 4173,
    proxy: {
      '/api': {
        target: apiTarget,
        changeOrigin: true,
      },
    },
  },
})


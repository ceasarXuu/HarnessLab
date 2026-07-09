/// <reference types="vite/client" />

declare module '*.css'

interface ImportMetaEnv {
  readonly VITE_ORNNLAB_DATA_MODE?: 'api' | 'mock'
}

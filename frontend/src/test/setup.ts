import { cleanup } from '@testing-library/react'
import '@testing-library/jest-dom/vitest'
import { ReadableStream, TransformStream, WritableStream } from 'node:stream/web'
import { afterEach } from 'vitest'

Object.assign(globalThis, { ReadableStream, TransformStream, WritableStream })

afterEach(() => cleanup())

/**
 * 轻量 async-state 原语 —— 不引入额外依赖。
 *
 * 错误抽象边界（BUG-WEB-04 R2）：
 * - ApiError（来自 apiClient）  → 网络/HTTP 层错误
 * - Error（原生）               → mapper 层业务错误
 * - AsyncState.error            → UI 统一错误态容器
 *
 * 归一化规则：View 的 fetcher try/catch 统一放入 AsyncState.error，
 * 不区分 ApiError 与原生 Error。StatePanel 渲染时按 instanceof 分流文案。
 */

import type { ApiError } from '@/api/client'

export type AsyncState<T, E = ApiError | Error> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'ready'; data: T }
  | { status: 'empty' }
  | { status: 'error'; error: E }

export const idle = <T>(): AsyncState<T> => ({ status: 'idle' })
export const loading = <T>(): AsyncState<T> => ({ status: 'loading' })
export const ready = <T>(data: T): AsyncState<T> => ({ status: 'ready', data })
export const empty = <T>(): AsyncState<T> => ({ status: 'empty' })
export const error = <T, E>(err: E): AsyncState<T, E> => ({ status: 'error', error: err })

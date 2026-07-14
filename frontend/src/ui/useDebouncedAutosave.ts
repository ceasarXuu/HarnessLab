import { useEffect, useRef } from 'react'

interface DebouncedAutosaveOptions<T> {
  delay?: number
  enabled?: boolean
  value: T
  onSave: (value: T) => boolean | Promise<boolean>
}

export function useDebouncedAutosave<T>({
  delay = 400,
  enabled = true,
  value,
  onSave,
}: DebouncedAutosaveOptions<T>) {
  const initialValue = useRef(JSON.stringify(value))
  const lastSavedValue = useRef(initialValue.current)
  const latestValue = useRef(value)
  const pendingValue = useRef<T | null>(null)
  const saveInFlight = useRef(false)
  const save = useRef(onSave)

  latestValue.current = value
  save.current = onSave

  useEffect(() => {
    if (!enabled) return undefined
    const serialized = JSON.stringify(value)
    if (serialized === lastSavedValue.current) return undefined

    const timer = window.setTimeout(() => {
      pendingValue.current = latestValue.current
      void flushPendingSave()
    }, delay)
    return () => window.clearTimeout(timer)
  }, [delay, enabled, value])

  const flushPendingSave = async () => {
    if (saveInFlight.current || !pendingValue.current) return
    const nextValue = pendingValue.current
    const serialized = JSON.stringify(nextValue)
    pendingValue.current = null
    if (serialized === lastSavedValue.current) return

    saveInFlight.current = true
    try {
      const saved = await save.current(nextValue)
      if (saved) lastSavedValue.current = serialized
    } finally {
      saveInFlight.current = false
      if (pendingValue.current) void flushPendingSave()
    }
  }
}

<script setup lang="ts" generic="T">
import type { AsyncState } from '@/utils/asyncState'
import { ApiError } from '@/api/client'

defineProps<{
  state: AsyncState<T>
  emptyMessage?: string
}>()

const emit = defineEmits<{
  retry: []
}>()

const errorSummary = (err: ApiError | Error): string => {
  if (err instanceof ApiError) {
    if (err.status === 404) return 'Resource not found'
    if (err.status >= 500) return 'Service temporarily unavailable'
    return `Request failed (HTTP ${err.status})`
  }
  return err.message || 'An unexpected error occurred'
}
</script>

<template>
  <div>
    <section v-if="state.status === 'loading'" class="state-panel state-panel--loading">
      <p class="muted">Loading…</p>
    </section>

    <section v-else-if="state.status === 'error'" class="state-panel state-panel--error">
      <p class="state-panel__message">{{ errorSummary(state.error) }}</p>
      <button class="btn" @click="emit('retry')">Retry</button>
    </section>

    <section v-else-if="state.status === 'empty'" class="state-panel state-panel--empty">
      <p class="muted">{{ emptyMessage ?? 'No data yet.' }}</p>
    </section>

    <section v-else-if="state.status === 'idle'" class="state-panel state-panel--idle">
      <p class="muted">Waiting…</p>
    </section>

    <slot v-else-if="state.status === 'ready'" :data="state.data" />
  </div>
</template>

<style scoped>
.state-panel {
  padding: var(--space-4);
}

.state-panel--loading,
.state-panel--idle {
  opacity: 0.65;
}

.state-panel--error {
  border-left: 3px solid var(--color-danger, #e03131);
  background: var(--color-danger-bg, rgba(224, 49, 49, 0.06));
}

.state-panel__message {
  font-weight: 600;
  margin-block-end: var(--space-2);
}

.btn {
  cursor: pointer;
}
</style>

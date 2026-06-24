<script setup lang="ts" generic="T">
import { useI18n } from 'vue-i18n'

import type { AsyncState } from '@/utils/asyncState'
import { ApiError } from '@/api/client'

defineProps<{
  state: AsyncState<T>
  emptyMessage?: string
}>()

const emit = defineEmits<{
  retry: []
}>()

const { t } = useI18n()

const errorSummary = (err: ApiError | Error): string => {
  if (err instanceof ApiError) {
    if (err.status === 404) return t('state.apiError404')
    if (err.status >= 500) return t('state.apiError5xx')
    return t('state.apiErrorOther', { status: err.status })
  }
  return err.message || t('state.unknown')
}
</script>

<template>
  <div>
    <section v-if="state.status === 'loading'" class="state-panel state-panel--loading">
      <p class="muted">{{ t('state.loading') }}</p>
    </section>

    <section v-else-if="state.status === 'error'" class="state-panel state-panel--error">
      <p class="state-panel__message">{{ errorSummary(state.error) }}</p>
      <button class="btn" @click="emit('retry')">{{ t('state.retry') }}</button>
    </section>

    <section v-else-if="state.status === 'empty'" class="state-panel state-panel--empty">
      <p class="muted">{{ emptyMessage ?? t('state.noData') }}</p>
    </section>

    <section v-else-if="state.status === 'idle'" class="state-panel state-panel--idle">
      <p class="muted">{{ t('state.waiting') }}</p>
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
  border-left: 3px solid var(--bad);
  background: var(--accent-soft);
}

.state-panel__message {
  font-weight: 600;
  margin-block-end: var(--space-2);
}

.btn {
  cursor: pointer;
}
</style>

<script setup lang="ts">
import type { KpiMetric } from '@/types/console'

defineProps<{
  metric: KpiMetric
}>()
</script>

<template>
  <article class="panel kpi-card">
    <div class="kpi-card__header">
      <p class="eyebrow">{{ metric.label }}</p>
      <span
        class="pill"
        :class="`pill--${metric.trend}`"
      >
        {{ metric.delta }}
      </span>
    </div>
    <strong class="kpi-card__value">{{ metric.value }}</strong>
    <p class="muted">{{ metric.description }}</p>
  </article>
</template>


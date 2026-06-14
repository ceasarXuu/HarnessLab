<script setup lang="ts">
import { rankLeaderboard } from '@/utils/leaderboard'
import type { ConsoleSnapshot } from '@/types/console'

const props = defineProps<{
  snapshot: ConsoleSnapshot
}>()

const ranked = rankLeaderboard(props.snapshot.leaderboard)
</script>

<template>
  <section class="page-grid">
    <section class="panel stack">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Benchmark posture</p>
          <h3>Score distribution</h3>
        </div>
      </div>
      <div class="table-list">
        <div class="table-list__head table-list__head--leaderboard">
          <span>Rank</span>
          <span>Agent</span>
          <span>Score</span>
          <span>Success</span>
          <span>Experiments</span>
        </div>
        <div
          v-for="entry in ranked"
          :key="entry.agent"
          class="table-list__row table-list__row--leaderboard"
        >
          <span>#{{ entry.rank }}</span>
          <span>{{ entry.agent }}</span>
          <span>{{ entry.score }}</span>
          <span>{{ Math.round(entry.successRate * 100) }}%</span>
          <span>{{ entry.experiments }}</span>
        </div>
      </div>
    </section>
  </section>
</template>

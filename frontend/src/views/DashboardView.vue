<script setup lang="ts">
import KpiCard from '@/components/KpiCard.vue'
import { rankLeaderboard } from '@/utils/leaderboard'
import type { ConsoleSnapshot } from '@/types/console'

const props = defineProps<{
  snapshot: ConsoleSnapshot
}>()

const ranked = rankLeaderboard(props.snapshot.leaderboard).slice(0, 3)
</script>

<template>
  <section class="page-grid">
    <section class="kpi-grid">
      <KpiCard
        v-for="metric in snapshot.metrics"
        :key="metric.label"
        :metric="metric"
      />
    </section>

    <section class="panel stack">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Priority alerts</p>
          <h3>Queue and reliability watchlist</h3>
        </div>
      </div>
      <article
        v-for="alert in snapshot.alerts"
        :key="alert.title"
        class="alert-row"
        :class="`alert-row--${alert.severity}`"
      >
        <div>
          <strong>{{ alert.title }}</strong>
          <p class="muted">{{ alert.detail }}</p>
        </div>
        <span class="pill">{{ alert.severity }}</span>
      </article>
    </section>

    <section class="panel stack">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Experiment focus</p>
          <h3>Current pipeline</h3>
        </div>
      </div>
      <div class="table-list">
        <div class="table-list__head">
          <span>Experiment</span>
          <span>State</span>
          <span>Owner</span>
          <span>Success</span>
        </div>
        <div
          v-for="experiment in snapshot.experiments"
          :key="experiment.id"
          class="table-list__row"
        >
          <span>
            <strong>{{ experiment.name }}</strong>
            <small>{{ experiment.target }}</small>
          </span>
          <span class="pill">{{ experiment.state }}</span>
          <span>{{ experiment.owner }}</span>
          <span>{{ experiment.successRate }}</span>
        </div>
      </div>
    </section>

    <section class="panel stack">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Top performers</p>
          <h3>Leaderboard snapshot</h3>
        </div>
      </div>
      <div class="leaderboard-list">
        <article
          v-for="entry in ranked"
          :key="entry.agent"
          class="leaderboard-list__row"
        >
          <div>
            <p class="eyebrow">Rank {{ entry.rank }}</p>
            <strong>{{ entry.agent }}</strong>
          </div>
          <div class="leaderboard-list__score">
            <strong>{{ entry.score }}</strong>
            <small>{{ Math.round(entry.successRate * 100) }}% success</small>
          </div>
        </article>
      </div>
    </section>
  </section>
</template>


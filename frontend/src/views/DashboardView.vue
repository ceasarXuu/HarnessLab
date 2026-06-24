<script setup lang="ts">
import { onMounted, ref } from 'vue'

import KpiCard from '@/components/KpiCard.vue'
import StatePanel from '@/components/StatePanel.vue'
import {
  ApiError,
  ornnLabApi,
  type Experiment,
  type LeaderboardEntryResponse,
} from '@/api/client'
import {
  toExperimentRecord,
  toKpiMetrics,
  toLeaderboardSeed,
} from '@/api/mappers'
import { rankLeaderboard } from '@/utils/leaderboard'
import { idle, loading, ready, empty, error, type AsyncState } from '@/utils/asyncState'
import type { ExperimentRecord, KpiMetric, LeaderboardEntry } from '@/types/console'

interface DashboardData {
  metrics: KpiMetric[]
  experiments: ExperimentRecord[]
  topAgents: LeaderboardEntry[]
}

const state = ref<AsyncState<DashboardData>>(idle())

const fetchDashboard = async () => {
  state.value = loading()
  try {
    const [exps, lb]: [Experiment[], LeaderboardEntryResponse[]] = await Promise.all([
      ornnLabApi.experiments(),
      ornnLabApi.leaderboard(),
    ])

    if (exps.length === 0 && lb.length === 0) {
      state.value = empty()
      return
    }

    state.value = ready<DashboardData>({
      metrics: toKpiMetrics(exps, lb),
      experiments: exps.map((e) => toExperimentRecord(e)),
      topAgents: rankLeaderboard(lb.map(toLeaderboardSeed)).slice(0, 3),
    })
  } catch (err) {
    state.value = error(err instanceof ApiError ? err : new Error(String(err)))
  }
}

onMounted(fetchDashboard)
</script>

<template>
  <StatePanel
    :state="state"
    empty-message="No experiments or leaderboard data yet."
    @retry="fetchDashboard"
  >
    <template #default="{ data }">
      <section class="page-grid">
        <section class="kpi-grid">
          <KpiCard
            v-for="metric in (data as DashboardData).metrics"
            :key="metric.label"
            :metric="metric"
          />
        </section>

        <!-- Priority alerts 区块按 BUG-WEB-02 处置表删除（无后端源） -->

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
              <span>Success</span>
            </div>
            <div
              v-for="experiment in (data as DashboardData).experiments"
              :key="experiment.id"
              class="table-list__row"
            >
              <span>
                <strong>{{ experiment.name }}</strong>
                <small>{{ experiment.target }}</small>
              </span>
              <span class="pill">{{ experiment.state }}</span>
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
              v-for="entry in (data as DashboardData).topAgents"
              :key="entry.agent"
              class="leaderboard-list__row"
            >
              <div>
                <p class="eyebrow">Rank {{ entry.rank }}</p>
                <strong>{{ entry.agent }}</strong>
              </div>
              <div class="leaderboard-list__score">
                <strong>{{ entry.score }}</strong>
              </div>
            </article>
          </div>
        </section>
      </section>
    </template>
  </StatePanel>
</template>

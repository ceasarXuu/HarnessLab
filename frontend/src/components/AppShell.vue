<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'

import { ApiError, ornnLabApi, type AgentResponse, type Experiment } from '@/api/client'

const route = useRoute()

const navItems = [
  { name: 'dashboard', label: 'Dashboard', to: '/' },
  { name: 'agents', label: 'Agents', to: '/agents' },
  { name: 'experiments', label: 'Experiments', to: '/experiments' },
  { name: 'leaderboard', label: 'Leaderboard', to: '/leaderboard' },
]

const experiments = ref<Experiment[]>([])
const agents = ref<AgentResponse[]>([])
const summaryError = ref<string | null>(null)

const pageTitle = computed(
  () => route.meta.title?.toString() ?? 'Operations Console',
)

const statusLine = computed(() => {
  if (summaryError.value) return summaryError.value
  const runningCount = experiments.value.filter((e) => e.status === 'running').length
  const blockedCount = agents.value.filter((a) => a.status !== 'compiled' && a.status !== 'draft').length

  return `${agents.value.length} agents live · ${runningCount} active experiments · ${blockedCount} blocked queues`
})

onMounted(async () => {
  try {
    // 并行拉两个端点；任一失败均退到摘要错误态，不阻塞子 view 自己的状态管理
    const [exps, ags] = await Promise.all([
      ornnLabApi.experiments(),
      ornnLabApi.agents.list(),
    ])
    experiments.value = exps
    agents.value = ags
  } catch (err) {
    if (err instanceof ApiError) {
      summaryError.value = `Live posture unavailable (HTTP ${err.status})`
    } else {
      summaryError.value = 'Live posture unavailable'
    }
  }
})
</script>

<template>
  <div class="console-shell">
    <aside class="console-shell__nav panel">
      <div>
        <p class="eyebrow">OrnnLab</p>
        <h1 class="nav-title">Operations Console</h1>
        <p class="muted">
          Focused front-end scaffold for operators managing agents, experiments,
          and benchmark performance.
        </p>
      </div>

      <nav
        class="nav-links"
        aria-label="Primary"
      >
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          :to="item.to"
          class="nav-link"
          active-class="nav-link--active"
        >
          {{ item.label }}
        </RouterLink>
      </nav>

      <section class="nav-summary">
        <p class="eyebrow">Live posture</p>
        <p class="muted">{{ statusLine }}</p>
      </section>
    </aside>

    <main class="console-shell__content">
      <header class="page-header panel">
        <div>
          <p class="eyebrow">Operator view</p>
          <h2>{{ pageTitle }}</h2>
        </div>
        <div class="page-header__meta">
          <span class="status-dot"></span>
          <span>API client primed for `/api`</span>
        </div>
      </header>

      <RouterView />
    </main>
  </div>
</template>

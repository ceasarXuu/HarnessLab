<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'

import type { ConsoleSnapshot } from '@/types/console'

const props = defineProps<{
  snapshot: ConsoleSnapshot
}>()

const route = useRoute()

const navItems = [
  { name: 'dashboard', label: 'Dashboard', to: '/' },
  { name: 'agents', label: 'Agents', to: '/agents' },
  { name: 'experiments', label: 'Experiments', to: '/experiments' },
  { name: 'leaderboard', label: 'Leaderboard', to: '/leaderboard' },
]

const pageTitle = computed(
  () => route.meta.title?.toString() ?? props.snapshot.headline,
)

const statusLine = computed(() => {
  const runningCount = props.snapshot.experiments.filter(
    (experiment) => experiment.state === 'running',
  ).length
  const blockedCount = props.snapshot.agents.filter(
    (agent) => agent.health === 'blocked',
  ).length

  return `${props.snapshot.agents.length} agents live · ${runningCount} active experiments · ${blockedCount} blocked queues`
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

      <RouterView v-slot="{ Component }">
        <component
          :is="Component"
          :snapshot="snapshot"
        />
      </RouterView>
    </main>
  </div>
</template>


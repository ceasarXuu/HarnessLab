<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { ApiError, ornnLabApi, type AgentResponse, type Experiment } from '@/api/client'

const route = useRoute()
const { t } = useI18n()

const navItems = [
  { name: 'dashboard', key: 'nav.dashboard', to: '/' },
  { name: 'agents', key: 'nav.agents', to: '/agents' },
  { name: 'experiments', key: 'nav.experiments', to: '/experiments' },
  { name: 'leaderboard', key: 'nav.leaderboard', to: '/leaderboard' },
] as const

const experiments = ref<Experiment[]>([])
const agents = ref<AgentResponse[]>([])
const summaryError = ref<{ http?: number } | null>(null)

const pageTitle = computed(() => {
  const key = route.meta.titleKey
  return key ? t(key) : t('app.subtitle')
})

const statusLine = computed(() => {
  if (summaryError.value) {
    return summaryError.value.http
      ? t('nav.postureUnavailableHttp', { status: summaryError.value.http })
      : t('nav.postureUnavailable')
  }
  const runningCount = experiments.value.filter((e) => e.status === 'running').length
  const blockedCount = agents.value.filter(
    (a) => a.status !== 'compiled' && a.status !== 'draft',
  ).length
  return t('nav.postureLine', {
    agents: agents.value.length,
    running: runningCount,
    blocked: blockedCount,
  })
})

onMounted(async () => {
  try {
    const [exps, ags] = await Promise.all([
      ornnLabApi.experiments(),
      ornnLabApi.agents.list(),
    ])
    experiments.value = exps
    agents.value = ags
  } catch (err) {
    summaryError.value = err instanceof ApiError ? { http: err.status } : {}
  }
})
</script>

<template>
  <div class="console-shell">
    <aside class="console-shell__nav panel">
      <div>
        <p class="eyebrow">{{ t('app.title') }}</p>
        <h1 class="nav-title">{{ t('app.subtitle') }}</h1>
        <p class="muted">{{ t('app.description') }}</p>
      </div>

      <nav
        class="nav-links"
        :aria-label="t('nav.primary')"
      >
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          :to="item.to"
          class="nav-link"
          active-class="nav-link--active"
        >
          {{ t(item.key) }}
        </RouterLink>
      </nav>

      <section class="nav-summary">
        <p class="eyebrow">{{ t('nav.livePosture') }}</p>
        <p class="muted">{{ statusLine }}</p>
      </section>
    </aside>

    <main class="console-shell__content">
      <header class="page-header panel">
        <div>
          <p class="eyebrow">{{ t('app.operatorView') }}</p>
          <h2>{{ pageTitle }}</h2>
        </div>
        <div class="page-header__meta">
          <span class="status-dot"></span>
          <span>{{ t('app.apiPrimed') }}</span>
        </div>
      </header>

      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { useLivePostureSummary } from '@/composables/useLivePostureSummary'

const route = useRoute()
const { t } = useI18n()
const { statusLine } = useLivePostureSummary()

const navItems = [
  { name: 'dashboard', key: 'nav.dashboard', to: '/' },
  { name: 'agents', key: 'nav.agents', to: '/agents' },
  { name: 'experiments', key: 'nav.experiments', to: '/experiments' },
  { name: 'leaderboard', key: 'nav.leaderboard', to: '/leaderboard' },
] as const

const pageTitle = computed(() => {
  const key = route.meta.titleKey
  return key ? t(key) : t('app.subtitle')
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

    <main
      id="main-content"
      class="console-shell__content"
      tabindex="-1"
    >
      <header class="page-header panel">
        <div>
          <p class="eyebrow">{{ t('app.operatorView') }}</p>
          <h2>{{ pageTitle }}</h2>
        </div>
        <div class="page-header__meta">
          <span class="status-dot" aria-hidden="true"></span>
          <span>{{ t('app.apiPrimed') }}</span>
        </div>
      </header>

      <RouterView />
    </main>
  </div>
</template>

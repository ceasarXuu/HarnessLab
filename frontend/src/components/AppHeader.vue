<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import LocaleToggle from './header/LocaleToggle.vue'
import ThemeToggle from './header/ThemeToggle.vue'
import GithubStars from './header/GithubStars.vue'

const { t } = useI18n()

// 仓库地址固定（与 git remote ceasarXuu/HarnessLab 对齐）；未来要走 env 再迁移
const GITHUB_REPO = 'ceasarXuu/HarnessLab'

const navItems = [
  { name: 'dashboard', key: 'nav.dashboard', to: '/' },
  { name: 'agents', key: 'nav.agents', to: '/agents' },
  { name: 'experiments', key: 'nav.experiments', to: '/experiments' },
  { name: 'leaderboard', key: 'nav.leaderboard', to: '/leaderboard' },
] as const
</script>

<template>
  <header class="app-header" role="banner">
    <div class="app-header__inner">
      <a class="app-header__brand" href="/">
        {{ t('app.title') }}
      </a>

      <nav class="app-header__nav" :aria-label="t('nav.primary')">
        <a
          v-for="item in navItems"
          :key="item.name"
          :href="item.to"
          class="app-header__nav-link"
        >
          {{ t(item.key) }}
        </a>
      </nav>

      <div class="app-header__controls">
        <GithubStars :repo="GITHUB_REPO" />
        <div class="app-header__mode-group">
          <LocaleToggle />
          <ThemeToggle />
        </div>
      </div>
    </div>
  </header>
</template>

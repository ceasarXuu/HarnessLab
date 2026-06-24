<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import LocaleToggle from './header/LocaleToggle.vue'
import ThemeToggle from './header/ThemeToggle.vue'
import GithubStars from './header/GithubStars.vue'

const { t } = useI18n()

// 仓库地址固定（与 git remote ceasarXuu/HarnessLab 对齐）；未来要走 env 再迁移
const GITHUB_REPO = 'ceasarXuu/HarnessLab'
</script>

<template>
  <header class="app-header" role="banner">
    <div class="app-header__brand">
      <span class="app-header__brand-dot" aria-hidden="true"></span>
      <span>{{ t('app.title') }}</span>
      <span class="muted">·</span>
      <span class="muted">{{ t('app.subtitle') }}</span>
    </div>

    <div class="app-header__controls">
      <LocaleToggle />
      <ThemeToggle />
      <GithubStars :repo="GITHUB_REPO" />
    </div>
  </header>
</template>

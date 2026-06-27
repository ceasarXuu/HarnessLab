<script setup lang="ts">
import { computed } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

const route = useRoute()
const { t } = useI18n()

const isHome = computed(() => route.name === 'dashboard')

const pageTitle = computed(() => {
  if (isHome.value) return t('app.title')
  const key = route.meta.titleKey
  return key ? t(key) : t('app.subtitle')
})
</script>

<template>
  <main
    id="main-content"
    class="hub-main"
    :class="{ 'hub-main--home': isHome }"
    tabindex="-1"
  >
    <header
      class="hub-page-heading"
      :class="{ 'hub-page-heading--home': isHome }"
    >
      <h1>{{ pageTitle }}</h1>
      <p v-if="isHome" class="muted">{{ t('app.description') }}</p>
    </header>

    <RouterView />
  </main>
</template>

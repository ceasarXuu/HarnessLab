<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { setLocale, type AppLocale } from '@/i18n'

const { t, locale } = useI18n()

const toggle = () => {
  const next: AppLocale = locale.value === 'zh' ? 'en' : 'zh'
  setLocale(next)
}
</script>

<template>
  <button
    type="button"
    class="header-control"
    :aria-label="t('header.toggleLanguage')"
    :title="t('header.toggleLanguage')"
    @click="toggle"
  >
    <span>{{ locale === 'zh' ? '中' : 'EN' }}</span>
  </button>
</template>

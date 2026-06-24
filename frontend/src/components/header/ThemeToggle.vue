<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useTheme } from '@/composables/useTheme'

const { t } = useI18n()
const { theme, toggleTheme } = useTheme()
</script>

<template>
  <button
    type="button"
    class="header-control"
    :aria-label="t('header.toggleTheme')"
    :title="t('header.toggleTheme')"
    @click="toggleTheme"
  >
    <span class="header-control__icon" aria-hidden="true">
      {{ theme === 'dark' ? '🌙' : '☀️' }}
    </span>
  </button>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import AppHeader from '@/components/AppHeader.vue'
import AppShell from '@/components/AppShell.vue'

const { t } = useI18n()
</script>

<template>
  <div class="app-layout">
    <a class="skip-link" href="#main-content">{{ t('app.skipToMain') }}</a>
    <AppHeader />
    <AppShell />
  </div>
</template>

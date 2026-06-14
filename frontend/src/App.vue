<script setup lang="ts">
import AppShell from '@/components/AppShell.vue'
import { buildConsoleSnapshot } from '@/data/consoleSnapshot'

const snapshot = buildConsoleSnapshot()
</script>

<template>
  <AppShell :snapshot="snapshot" />
</template>


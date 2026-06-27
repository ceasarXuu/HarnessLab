<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useGithubStars, formatStars } from '@/composables/useGithubStars'

const props = defineProps<{
  /** owner/repo，例如 "ceasarXuu/HarnessLab" */
  repo: string
}>()

const { t } = useI18n()
const { stars, loading, fetchStars } = useGithubStars(props.repo)

onMounted(fetchStars)

const href = computed(() => `https://github.com/${props.repo}`)
const displayCount = computed(() =>
  stars.value === null ? '—' : formatStars(stars.value),
)
const ariaLabel = computed(() =>
  t('header.githubAria', { count: stars.value ?? '?' }),
)
</script>

<template>
  <a
    class="header-control header-control--primary"
    :href="href"
    target="_blank"
    rel="noopener noreferrer"
    :aria-label="ariaLabel"
    :aria-busy="loading"
    :title="ariaLabel"
  >
    <span class="header-control__icon" aria-hidden="true">GitHub</span>
    <span class="header-control__count">{{ displayCount }}</span>
    <span class="visually-hidden">{{ t('header.githubLabel') }}</span>
  </a>
</template>

<style scoped>
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
</style>

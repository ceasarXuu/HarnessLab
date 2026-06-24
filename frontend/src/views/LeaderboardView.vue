<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import StatePanel from '@/components/StatePanel.vue'
import { ApiError, ornnLabApi, type LeaderboardEntryResponse } from '@/api/client'
import { toLeaderboardSeed } from '@/api/mappers'
import { rankLeaderboard } from '@/utils/leaderboard'
import { idle, loading, ready, empty, error, type AsyncState } from '@/utils/asyncState'
import type { LeaderboardEntry } from '@/types/console'

const { t } = useI18n()

const state = ref<AsyncState<LeaderboardEntry[]>>(idle())

const fetchLeaderboard = async () => {
  state.value = loading()
  try {
    const raw: LeaderboardEntryResponse[] = await ornnLabApi.leaderboard()
    if (raw.length === 0) {
      state.value = empty()
      return
    }
    state.value = ready(rankLeaderboard(raw.map(toLeaderboardSeed)))
  } catch (err) {
    state.value = error(err instanceof ApiError ? err : new Error(String(err)))
  }
}

onMounted(fetchLeaderboard)
</script>

<template>
  <StatePanel
    :state="state"
    :empty-message="t('empty.leaderboard')"
    @retry="fetchLeaderboard"
  >
    <template #default="{ data }">
      <section class="page-grid">
        <section class="panel stack">
          <div class="section-heading">
            <div>
              <p class="eyebrow">{{ t('leaderboard.postureEyebrow') }}</p>
              <h3>{{ t('leaderboard.postureTitle') }}</h3>
            </div>
          </div>
          <!-- BUG-WEB-02 处置：Success / Experiments 列移除（mapper 派生为 0，无意义） -->
          <div class="table-list">
            <div class="table-list__head table-list__head--leaderboard">
              <span>{{ t('leaderboard.headRank') }}</span>
              <span>{{ t('leaderboard.headAgent') }}</span>
              <span>{{ t('leaderboard.headScore') }}</span>
            </div>
            <div
              v-for="entry in (data as LeaderboardEntry[])"
              :key="entry.agent"
              class="table-list__row table-list__row--leaderboard"
            >
              <span>#{{ entry.rank }}</span>
              <span>{{ entry.agent }}</span>
              <span>{{ entry.score }}</span>
            </div>
          </div>
        </section>
      </section>
    </template>
  </StatePanel>
</template>

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
          <table class="data-table data-table--leaderboard">
            <thead>
              <tr>
                <th scope="col">{{ t('leaderboard.headRank') }}</th>
                <th scope="col">{{ t('leaderboard.headAgent') }}</th>
                <th scope="col">{{ t('leaderboard.headScore') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="entry in (data as LeaderboardEntry[])"
                :key="entry.agent"
              >
                <td>#{{ entry.rank }}</td>
                <td>{{ entry.agent }}</td>
                <td class="data-table__numeric">{{ entry.score }}</td>
              </tr>
            </tbody>
          </table>
        </section>
      </section>
    </template>
  </StatePanel>
</template>

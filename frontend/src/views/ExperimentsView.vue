<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import StatePanel from '@/components/StatePanel.vue'
import { ApiError, ornnLabApi, type Experiment } from '@/api/client'
import { toExperimentRecord } from '@/api/mappers'
import { idle, loading, ready, empty, error, type AsyncState } from '@/utils/asyncState'
import type { ExperimentRecord } from '@/types/console'

const { t } = useI18n()

const state = ref<AsyncState<ExperimentRecord[]>>(idle())

const fetchExperiments = async () => {
  state.value = loading()
  try {
    const raw: Experiment[] = await ornnLabApi.experiments()
    if (raw.length === 0) {
      state.value = empty()
      return
    }
    state.value = ready(raw.map((e) => toExperimentRecord(e)))
  } catch (err) {
    state.value = error(err instanceof ApiError ? err : new Error(String(err)))
  }
}

onMounted(fetchExperiments)
</script>

<template>
  <StatePanel
    :state="state"
    :empty-message="t('empty.experiments')"
    @retry="fetchExperiments"
  >
    <template #default="{ data }">
      <section class="page-grid">
        <section class="panel stack">
          <div class="section-heading">
            <div>
              <p class="eyebrow">{{ t('experiments.ledgerEyebrow') }}</p>
              <h3>{{ t('experiments.ledgerTitle') }}</h3>
            </div>
            <p class="muted">{{ t('experiments.ledgerHint') }}</p>
          </div>

          <!-- BUG-WEB-02 处置：删除 Owner 行（无后端源） -->
          <div class="experiment-grid">
            <article
              v-for="experiment in (data as ExperimentRecord[])"
              :key="experiment.id"
              class="experiment-card"
            >
              <div class="experiment-card__header">
                <div>
                  <p class="eyebrow">{{ experiment.id }}</p>
                  <h4>{{ experiment.name }}</h4>
                </div>
                <span class="pill">{{ experiment.state }}</span>
              </div>
              <dl class="detail-grid">
                <div>
                  <dt>{{ t('experiments.target') }}</dt>
                  <dd>{{ experiment.target }}</dd>
                </div>
                <div>
                  <dt>{{ t('experiments.updated') }}</dt>
                  <dd>{{ experiment.updatedAt }}</dd>
                </div>
                <div>
                  <dt>{{ t('experiments.success') }}</dt>
                  <dd>{{ experiment.successRate }}</dd>
                </div>
              </dl>
            </article>
          </div>
        </section>
      </section>
    </template>
  </StatePanel>
</template>

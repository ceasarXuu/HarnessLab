<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import StatePanel from '@/components/StatePanel.vue'
import {
  ApiError,
  ornnLabApi,
  type Experiment,
} from '@/api/client'
import { idle, loading, ready, empty, error, type AsyncState } from '@/utils/asyncState'

const { t } = useI18n()

interface DatasetRow {
  name: string
  tasks: number
}

interface DashboardData {
  datasets: DatasetRow[]
}

const state = ref<AsyncState<DashboardData>>(idle())

const fetchDashboard = async () => {
  state.value = loading()
  try {
    const exps: Experiment[] = await ornnLabApi.experiments()

    if (exps.length === 0) {
      state.value = empty()
      return
    }

    state.value = ready<DashboardData>({
      datasets: exps.map((experiment) => ({
        name: experiment.name,
        tasks: experiment.requested_run_count,
      })),
    })
  } catch (err) {
    state.value = error(err instanceof ApiError ? err : new Error(String(err)))
  }
}

const copyText = async (value: string) => {
  await navigator.clipboard?.writeText(value)
}

onMounted(fetchDashboard)
</script>

<template>
  <StatePanel
    :state="state"
    :empty-message="t('empty.dashboard')"
    @retry="fetchDashboard"
  >
    <template #default="{ data }">
      <div class="home-table">
        <table class="data-table data-table--datasets">
          <colgroup>
            <col class="data-table__dataset-col" />
            <col class="data-table__tasks-col" />
          </colgroup>
          <thead>
            <tr>
              <th scope="col">{{ t('table.dataset') }}</th>
              <th scope="col">{{ t('table.tasks') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="dataset in (data as DashboardData).datasets"
              :key="dataset.name"
            >
              <td>
                <span class="dataset-cell">
                  <span>{{ dataset.name }}</span>
                  <button
                    type="button"
                    class="copy-button"
                    :aria-label="t('dashboard.copyDataset', { name: dataset.name })"
                    @click="copyText(dataset.name)"
                  >
                    ⧉
                  </button>
                </span>
              </td>
              <td class="data-table__numeric">{{ dataset.tasks.toLocaleString() }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </StatePanel>

  <section class="publish-section" aria-labelledby="publish-title">
    <div class="publish-section__heading">
      <h2 id="publish-title">{{ t('dashboard.publishTitle') }}</h2>
      <p class="muted">
        {{ t('dashboard.publishDescription') }} <code>/publish</code>.
      </p>
    </div>

    <div class="code-block">
      <div class="code-block__header">
        <span>{{ t('dashboard.installSkillTitle') }}</span>
      </div>
      <pre><code>npx skills add harbor-framework/harbor --skill publish</code></pre>
    </div>

    <div class="code-block">
      <div class="code-block__header">
        <span>{{ t('dashboard.runPublishTitle') }}</span>
      </div>
      <pre><code>/publish</code></pre>
    </div>
  </section>
</template>

import { computed, onMounted, readonly, shallowRef } from 'vue'
import { useI18n } from 'vue-i18n'

import { ApiError, ornnLabApi, type AgentResponse, type Experiment } from '@/api/client'

type SummaryError = { http?: number }

export const useLivePostureSummary = () => {
  const { t } = useI18n()
  const experiments = shallowRef<Experiment[]>([])
  const agents = shallowRef<AgentResponse[]>([])
  const summaryError = shallowRef<SummaryError | null>(null)

  const fetchSummary = async () => {
    summaryError.value = null
    try {
      const [exps, ags] = await Promise.all([
        ornnLabApi.experiments(),
        ornnLabApi.agents.list(),
      ])
      experiments.value = exps
      agents.value = ags
    } catch (err) {
      summaryError.value = err instanceof ApiError ? { http: err.status } : {}
    }
  }

  const statusLine = computed(() => {
    if (summaryError.value) {
      return summaryError.value.http
        ? t('nav.postureUnavailableHttp', { status: summaryError.value.http })
        : t('nav.postureUnavailable')
    }

    const runningCount = experiments.value.filter((e) => e.status === 'running').length
    const blockedCount = agents.value.filter(
      (a) => a.status !== 'compiled' && a.status !== 'draft',
    ).length

    return t('nav.postureLine', {
      agents: agents.value.length,
      running: runningCount,
      blocked: blockedCount,
    })
  })

  onMounted(fetchSummary)

  return {
    experiments: readonly(experiments),
    agents: readonly(agents),
    summaryError: readonly(summaryError),
    statusLine,
    fetchSummary,
  }
}

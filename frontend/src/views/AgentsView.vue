<script setup lang="ts">
import { onMounted, ref } from 'vue'

import StatePanel from '@/components/StatePanel.vue'
import { ApiError, ornnLabApi, type AgentResponse } from '@/api/client'
import { toAgentRecord } from '@/api/mappers'
import { idle, loading, ready, empty, error, type AsyncState } from '@/utils/asyncState'
import type { AgentRecord } from '@/types/console'

interface AgentsData {
  agents: AgentRecord[]
  counts: { healthy: number; warming: number; blocked: number }
}

const state = ref<AsyncState<AgentsData>>(idle())

const fetchAgents = async () => {
  state.value = loading()
  try {
    const raw: AgentResponse[] = await ornnLabApi.agents.list()
    if (raw.length === 0) {
      state.value = empty()
      return
    }

    const agents = raw.map((a) => toAgentRecord(a))
    state.value = ready<AgentsData>({
      agents,
      counts: {
        healthy: agents.filter((a) => a.health === 'healthy').length,
        warming: agents.filter((a) => a.health === 'warming').length,
        blocked: agents.filter((a) => a.health === 'blocked').length,
      },
    })
  } catch (err) {
    state.value = error(err instanceof ApiError ? err : new Error(String(err)))
  }
}

onMounted(fetchAgents)
</script>

<template>
  <StatePanel
    :state="state"
    empty-message="No agents registered yet."
    @retry="fetchAgents"
  >
    <template #default="{ data }">
      <section class="page-grid">
        <section class="panel stack">
          <div class="section-heading">
            <div>
              <p class="eyebrow">Agent posture</p>
              <h3>Fleet overview</h3>
            </div>
          </div>
          <div class="summary-grid">
            <article class="summary-card">
              <span class="eyebrow">Healthy</span>
              <strong>{{ (data as AgentsData).counts.healthy }}</strong>
            </article>
            <article class="summary-card">
              <span class="eyebrow">Warming</span>
              <strong>{{ (data as AgentsData).counts.warming }}</strong>
            </article>
            <article class="summary-card">
              <span class="eyebrow">Blocked</span>
              <strong>{{ (data as AgentsData).counts.blocked }}</strong>
            </article>
          </div>
        </section>

        <section class="panel stack">
          <div class="section-heading">
            <div>
              <p class="eyebrow">Fleet details</p>
              <h3>Operators</h3>
            </div>
          </div>
          <!-- BUG-WEB-02 处置：删除 Queue / Heartbeat 列、owner subtitle（无后端源） -->
          <div class="table-list">
            <div class="table-list__head table-list__head--agents">
              <span>Agent</span>
              <span>Health</span>
              <span>Runs</span>
              <span>Success</span>
            </div>
            <div
              v-for="agent in (data as AgentsData).agents"
              :key="agent.name"
              class="table-list__row table-list__row--agents"
            >
              <span>
                <strong>{{ agent.name }}</strong>
              </span>
              <span class="pill">{{ agent.health }}</span>
              <span>{{ agent.activeRuns }}</span>
              <span>{{ agent.successRate }}</span>
            </div>
          </div>
        </section>
      </section>
    </template>
  </StatePanel>
</template>

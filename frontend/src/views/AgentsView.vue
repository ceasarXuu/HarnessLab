<script setup lang="ts">
import type { ConsoleSnapshot } from '@/types/console'

const props = defineProps<{
  snapshot: ConsoleSnapshot
}>()

const statusCounts = {
  healthy: props.snapshot.agents.filter((agent) => agent.health === 'healthy').length,
  warming: props.snapshot.agents.filter((agent) => agent.health === 'warming').length,
  blocked: props.snapshot.agents.filter((agent) => agent.health === 'blocked').length,
}
</script>

<template>
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
          <strong>{{ statusCounts.healthy }}</strong>
        </article>
        <article class="summary-card">
          <span class="eyebrow">Warming</span>
          <strong>{{ statusCounts.warming }}</strong>
        </article>
        <article class="summary-card">
          <span class="eyebrow">Blocked</span>
          <strong>{{ statusCounts.blocked }}</strong>
        </article>
      </div>
    </section>

    <section class="panel stack">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Fleet details</p>
          <h3>Operators by queue</h3>
        </div>
      </div>
      <div class="table-list">
        <div class="table-list__head table-list__head--agents">
          <span>Agent</span>
          <span>Queue</span>
          <span>Health</span>
          <span>Runs</span>
          <span>Heartbeat</span>
          <span>Success</span>
        </div>
        <div
          v-for="agent in snapshot.agents"
          :key="agent.name"
          class="table-list__row table-list__row--agents"
        >
          <span>
            <strong>{{ agent.name }}</strong>
            <small>{{ agent.owner }}</small>
          </span>
          <span>{{ agent.queue }}</span>
          <span class="pill">{{ agent.health }}</span>
          <span>{{ agent.activeRuns }}</span>
          <span>{{ agent.lastHeartbeat }}</span>
          <span>{{ agent.successRate }}</span>
        </div>
      </div>
    </section>
  </section>
</template>

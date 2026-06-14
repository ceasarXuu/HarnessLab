<script setup lang="ts">
import type { ConsoleSnapshot } from '@/types/console'

defineProps<{
  snapshot: ConsoleSnapshot
}>()
</script>

<template>
  <section class="page-grid">
    <section class="panel stack">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Experiment ledger</p>
          <h3>Execution lanes</h3>
        </div>
        <p class="muted">
          Queue-focused view for tracking benchmark targets, ownership, and
          recent movement.
        </p>
      </div>

      <div class="experiment-grid">
        <article
          v-for="experiment in snapshot.experiments"
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
              <dt>Owner</dt>
              <dd>{{ experiment.owner }}</dd>
            </div>
            <div>
              <dt>Target</dt>
              <dd>{{ experiment.target }}</dd>
            </div>
            <div>
              <dt>Updated</dt>
              <dd>{{ experiment.updatedAt }}</dd>
            </div>
            <div>
              <dt>Success</dt>
              <dd>{{ experiment.successRate }}</dd>
            </div>
          </dl>
        </article>
      </div>
    </section>
  </section>
</template>

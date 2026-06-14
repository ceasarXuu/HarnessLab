import { createRouter, createWebHistory } from 'vue-router'

import AgentsView from '@/views/AgentsView.vue'
import DashboardView from '@/views/DashboardView.vue'
import ExperimentsView from '@/views/ExperimentsView.vue'
import LeaderboardView from '@/views/LeaderboardView.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView,
      meta: { label: 'Dashboard', title: 'Operations Dashboard' },
    },
    {
      path: '/agents',
      name: 'agents',
      component: AgentsView,
      meta: { label: 'Agents', title: 'Agent Fleet' },
    },
    {
      path: '/experiments',
      name: 'experiments',
      component: ExperimentsView,
      meta: { label: 'Experiments', title: 'Experiment Pipeline' },
    },
    {
      path: '/leaderboard',
      name: 'leaderboard',
      component: LeaderboardView,
      meta: { label: 'Leaderboard', title: 'Benchmark Leaderboard' },
    },
  ],
})


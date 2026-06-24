import { createRouter, createWebHistory } from 'vue-router'

import AgentsView from '@/views/AgentsView.vue'
import DashboardView from '@/views/DashboardView.vue'
import ExperimentsView from '@/views/ExperimentsView.vue'
import LeaderboardView from '@/views/LeaderboardView.vue'

declare module 'vue-router' {
  interface RouteMeta {
    label?: string
    /** i18n key 用于 AppShell 标题 */
    titleKey?: string
  }
}

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView,
      meta: { label: 'Dashboard', titleKey: 'pageTitle.dashboard' },
    },
    {
      path: '/agents',
      name: 'agents',
      component: AgentsView,
      meta: { label: 'Agents', titleKey: 'pageTitle.agents' },
    },
    {
      path: '/experiments',
      name: 'experiments',
      component: ExperimentsView,
      meta: { label: 'Experiments', titleKey: 'pageTitle.experiments' },
    },
    {
      path: '/leaderboard',
      name: 'leaderboard',
      component: LeaderboardView,
      meta: { label: 'Leaderboard', titleKey: 'pageTitle.leaderboard' },
    },
  ],
})

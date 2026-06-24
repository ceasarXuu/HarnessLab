/**
 * View-model mapper —— 将后端 Response 类型映射为 UI 使用的 viewmodel。
 *
 * 判据（BUG-WEB-03 R3）：
 * - 1:1 字段复制 → 在 View 层直接消费，不引入 mapper
 * - 枚举映射 / 重命名 / 聚合 / 多对一派生 → 由本文件 mapper 负责
 *
 * "删除"字段约定（BUG-WEB-03 N3）：
 * - 无后端源的字段填充空值（"" / 0 / []），保持 types/console.ts 类型不变
 * - 真正从 types/console.ts 移除字段留到 v0.1.5（与 OpenAPI 自动类型生成一起评估）
 */

import type { Experiment, ExperimentRun, AgentResponse, LeaderboardEntryResponse } from './client'
import type {
  ExperimentRecord,
  AgentRecord,
  KpiMetric,
  LeaderboardSeed,
} from '@/types/console'

// ─── helpers ────────────────────────────────────────────────

const mapExperimentState = (status: string): ExperimentRecord['state'] => {
  if (status === 'completed') return 'complete'
  if (status === 'queued' || status === 'draft') return 'queued'
  return 'running' // failed / interrupted / cancelled 均归为 running
}

const calcSuccessRate = (runs: ExperimentRun[]): string => {
  if (runs.length === 0) return '—'
  const completed = runs.filter((r) => r.status === 'completed').length
  return `${Math.round((completed / runs.length) * 100)}%`
}

const mapAgentHealth = (status: string): AgentRecord['health'] => {
  if (status === 'compiled') return 'healthy'
  if (status === 'draft') return 'warming'
  return 'blocked' // deleted / error / unknown
}

// ─── ExperimentRecord ───────────────────────────────────────

export const toExperimentRecord = (
  experiment: Experiment,
  runs?: ExperimentRun[],
): ExperimentRecord => ({
  id: experiment.id,
  name: experiment.name,
  owner: '', // 无后端源（N3：删除字段 → 填 ""）
  state: mapExperimentState(experiment.status),
  target: experiment.kind,
  updatedAt: experiment.updated_at,
  successRate: runs ? calcSuccessRate(runs) : '—',
})

// ─── AgentRecord ─────────────────────────────────────────────

export const toAgentRecord = (
  agent: AgentResponse,
  activeRuns = 0,
): AgentRecord => ({
  name: agent.name,
  owner: '', // 无后端源（N3）
  queue: '', // 无后端源（N3）
  health: mapAgentHealth(agent.status),
  activeRuns,
  lastHeartbeat: '', // 无后端源（N3）
  successRate: '—', // 无直接后端源，本期先固定
})

// ─── KpiMetric ───────────────────────────────────────────────

export const toKpiMetrics = (
  experiments: Experiment[],
  leaderboard: LeaderboardEntryResponse[],
): KpiMetric[] => {
  const totalExps = experiments.length
  const running = experiments.filter((e) => e.status === 'running').length
  const bestScore = leaderboard.reduce((max, e) =>
    (e.score ?? 0) > max ? (e.score ?? 0) : max, 0)

  return [
    {
      label: 'Total experiments',
      value: String(totalExps),
      delta: String(running) + ' running',
      trend: running > 0 ? 'up' : 'neutral',
      description: 'All experiments in the local workspace.',
    },
    {
      label: 'Best leaderboard score',
      value: bestScore > 0 ? bestScore.toFixed(2) : '—',
      delta: '',
      trend: 'neutral',
      description: 'Highest score across all benchmarks on the leaderboard.',
    },
    {
      label: 'Active agents',
      value: '—',
      delta: '',
      trend: 'neutral',
      description: 'Agent count requires system/status aggregation; pending.',
    },
    {
      label: 'Experiments today',
      value: '—',
      delta: '',
      trend: 'neutral',
      description: 'Time-series aggregation pending; not in scope.',
    },
  ]
}

// ─── LeaderboardSeed ────────────────────────────────────────

export const toLeaderboardSeed = (
  entry: LeaderboardEntryResponse,
): LeaderboardSeed => ({
  agent: entry.agent_id,
  score: entry.score ?? 0,
  successRate: 0, // 无直接后端源，本期先固定
  experiments: 0, // 无直接后端源，本期先固定
})

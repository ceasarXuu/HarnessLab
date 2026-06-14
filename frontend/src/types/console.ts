export type MetricTrend = 'down' | 'neutral' | 'up'
export type AgentHealth = 'blocked' | 'healthy' | 'warming'
export type ExperimentState = 'complete' | 'queued' | 'running'
export type Severity = 'high' | 'medium' | 'low'

export interface KpiMetric {
  label: string
  value: string
  delta: string
  trend: MetricTrend
  description: string
}

export interface AlertItem {
  title: string
  detail: string
  severity: Severity
}

export interface AgentRecord {
  name: string
  owner: string
  queue: string
  health: AgentHealth
  activeRuns: number
  lastHeartbeat: string
  successRate: string
}

export interface ExperimentRecord {
  id: string
  name: string
  owner: string
  state: ExperimentState
  target: string
  updatedAt: string
  successRate: string
}

export interface LeaderboardSeed {
  agent: string
  score: number
  successRate: number
  experiments: number
}

export interface LeaderboardEntry extends LeaderboardSeed {
  rank: number
}

export interface ConsoleSnapshot {
  headline: string
  metrics: KpiMetric[]
  alerts: AlertItem[]
  agents: AgentRecord[]
  experiments: ExperimentRecord[]
  leaderboard: LeaderboardSeed[]
}


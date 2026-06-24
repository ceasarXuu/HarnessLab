import { describe, it, expect } from 'vitest'
import {
  toExperimentRecord,
  toAgentRecord,
  toKpiMetrics,
  toLeaderboardSeed,
} from './mappers'
import type { Experiment, ExperimentRun, AgentResponse, LeaderboardEntryResponse } from './client'

// ─── toExperimentRecord ────────────────────────────────────

describe('toExperimentRecord', () => {
  const base: Experiment = {
    id: 'exp-1',
    name: 'baseline-eval',
    kind: 'batch',
    status: 'completed',
    requested_run_count: 10,
    mode: 'terminal-bench',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-06-01T00:00:00Z',
  }

  it('maps 1:1 fields: id, name', () => {
    const r = toExperimentRecord(base)
    expect(r.id).toBe('exp-1')
    expect(r.name).toBe('baseline-eval')
  })

  it('remaps kind → target', () => {
    expect(toExperimentRecord(base).target).toBe('batch')
  })

  it('remaps updated_at → updatedAt', () => {
    expect(toExperimentRecord(base).updatedAt).toBe('2026-06-01T00:00:00Z')
  })

  it('maps status completed → complete', () => {
    expect(toExperimentRecord({ ...base, status: 'completed' }).state).toBe('complete')
  })

  it('maps status queued/draft → queued', () => {
    expect(toExperimentRecord({ ...base, status: 'queued' }).state).toBe('queued')
    expect(toExperimentRecord({ ...base, status: 'draft' }).state).toBe('queued')
  })

  it('maps status running/failed/interrupted/cancelled → running', () => {
    for (const s of ['running', 'failed', 'interrupted', 'cancelled'] as const) {
      expect(toExperimentRecord({ ...base, status: s }).state).toBe('running')
    }
  })

  it('computes successRate from runs', () => {
    const runs: ExperimentRun[] = [
      makeRun('1', 'completed'),
      makeRun('2', 'completed'),
      makeRun('3', 'failed'),
    ]
    expect(toExperimentRecord(base, runs).successRate).toBe('67%')
  })

  it('returns "—" when runs is undefined or empty', () => {
    expect(toExperimentRecord(base).successRate).toBe('—')
    expect(toExperimentRecord(base, []).successRate).toBe('—')
  })

  it('returns 0% when all runs failed', () => {
    const runs = [makeRun('1', 'failed'), makeRun('2', 'failed')]
    expect(toExperimentRecord(base, runs).successRate).toBe('0%')
  })

  it('fills owner with "" (N3)', () => {
    expect(toExperimentRecord(base).owner).toBe('')
  })
})

// ─── toAgentRecord ─────────────────────────────────────────

describe('toAgentRecord', () => {
  const base: AgentResponse = {
    id: 'agt-1',
    name: 'harbor-v1',
    kind: 'harbor',
    harbor_agent_name: 'harbor/agent',
    harbor_import_path: null,
    model_name: 'gpt-4',
    status: 'compiled',
    profile_path: '/tmp/agt.json',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-06-01T00:00:00Z',
  }

  it('maps name directly', () => {
    expect(toAgentRecord(base).name).toBe('harbor-v1')
  })

  it('maps status compiled → healthy', () => {
    expect(toAgentRecord({ ...base, status: 'compiled' }).health).toBe('healthy')
  })

  it('maps status draft → warming', () => {
    expect(toAgentRecord({ ...base, status: 'draft' }).health).toBe('warming')
  })

  it('maps status deleted/error/unknown → blocked', () => {
    expect(toAgentRecord({ ...base, status: 'deleted' }).health).toBe('blocked')
    expect(toAgentRecord({ ...base, status: 'error' }).health).toBe('blocked')
    expect(toAgentRecord({ ...base, status: 'unknown' }).health).toBe('blocked')
  })

  it('fills owner / queue / lastHeartbeat with "" (N3)', () => {
    const r = toAgentRecord(base)
    expect(r.owner).toBe('')
    expect(r.queue).toBe('')
    expect(r.lastHeartbeat).toBe('')
  })

  it('uses activeRuns param', () => {
    expect(toAgentRecord(base, 3).activeRuns).toBe(3)
    expect(toAgentRecord(base).activeRuns).toBe(0)
  })

  it('fills successRate with "—"', () => {
    expect(toAgentRecord(base).successRate).toBe('—')
  })
})

// ─── toKpiMetrics ──────────────────────────────────────────

describe('toKpiMetrics', () => {
  const exps: Experiment[] = [
    { id: '1', name: 'a', kind: 'batch', status: 'running', requested_run_count: 1, mode: 'x', created_at: '', updated_at: '' },
    { id: '2', name: 'b', kind: 'batch', status: 'completed', requested_run_count: 1, mode: 'x', created_at: '', updated_at: '' },
    { id: '3', name: 'c', kind: 'batch', status: 'completed', requested_run_count: 1, mode: 'x', created_at: '', updated_at: '' },
  ]

  it('returns 4 metrics', () => {
    const metrics = toKpiMetrics(exps, [])
    expect(metrics).toHaveLength(4)
  })

  it('counts total experiments', () => {
    const metrics = toKpiMetrics(exps, [])
    expect(metrics[0].value).toBe('3')
  })

  it('counts running experiments as delta', () => {
    const metrics = toKpiMetrics(exps, [])
    expect(metrics[0].delta).toBe('1 running')
    expect(metrics[0].trend).toBe('up')
  })

  it('computes best leaderboard score', () => {
    const lb: LeaderboardEntryResponse[] = [
      makeLBEntry('a1', 'bench', 0.85),
      makeLBEntry('a2', 'bench', 0.92),
      makeLBEntry('a3', 'bench', 0.78),
    ]
    const metrics = toKpiMetrics(exps, lb)
    expect(metrics[1].value).toBe('0.92')
  })

  it('returns "—" when leaderboard is empty', () => {
    const metrics = toKpiMetrics(exps, [])
    expect(metrics[1].value).toBe('—')
  })
})

// ─── toLeaderboardSeed ─────────────────────────────────────

describe('toLeaderboardSeed', () => {
  it('re-maps agent_id → agent', () => {
    const entry = makeLBEntry('agt-1', 'bench', 0.95)
    expect(toLeaderboardSeed(entry).agent).toBe('agt-1')
  })

  it('maps score', () => {
    expect(toLeaderboardSeed(makeLBEntry('a', 'b', 0.88)).score).toBe(0.88)
  })

  it('uses 0 when score is null', () => {
    const entry = { ...makeLBEntry('a', 'b', 0.95), score: null as unknown as number }
    expect(toLeaderboardSeed(entry).score).toBe(0)
  })

  it('fills successRate / experiments with 0 (N3)', () => {
    const r = toLeaderboardSeed(makeLBEntry('a', 'b', 0.5))
    expect(r.successRate).toBe(0)
    expect(r.experiments).toBe(0)
  })
})

// ─── helpers ────────────────────────────────────────────────

function makeRun(id: string, status: ExperimentRun['status']): ExperimentRun {
  return {
    id,
    experiment_id: 'exp-1',
    status,
    run_order: 1,
    agent_id: 'a1',
    benchmark_name: 'bench',
    benchmark_version: null,
    split: null,
    n_tasks: null,
    n_attempts: 1,
    n_concurrent: 1,
    score: status === 'completed' ? 0.9 : null,
    job_dir: null,
    report_path: null,
    failure_class: null,
    failure_code: null,
  }
}

function makeLBEntry(
  agentId: string,
  benchmark: string,
  score: number,
): LeaderboardEntryResponse {
  return {
    id: `${agentId}-${benchmark}`,
    agent_id: agentId,
    benchmark_name: benchmark,
    benchmark_version: null,
    split: null,
    finished_at: null,
    score,
    comparability_key: `${agentId}-${benchmark}`,
    report_path: null,
  }
}

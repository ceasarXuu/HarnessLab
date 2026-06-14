import { rankLeaderboard } from './leaderboard'

describe('rankLeaderboard', () => {
  it('sorts entries by score, success rate, then agent name and assigns ranks', () => {
    const ranked = rankLeaderboard([
      { agent: 'Zephyr', score: 82, successRate: 0.91, experiments: 18 },
      { agent: 'Aegis', score: 91, successRate: 0.93, experiments: 21 },
      { agent: 'Beacon', score: 91, successRate: 0.91, experiments: 19 },
    ])

    expect(ranked.map((entry) => `${entry.rank}:${entry.agent}`)).toEqual([
      '1:Aegis',
      '2:Beacon',
      '3:Zephyr',
    ])
  })
})


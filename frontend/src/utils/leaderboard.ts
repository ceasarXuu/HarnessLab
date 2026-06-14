import type { LeaderboardEntry, LeaderboardSeed } from '@/types/console'

const compareEntries = (left: LeaderboardSeed, right: LeaderboardSeed) => {
  if (left.score !== right.score) {
    return right.score - left.score
  }

  if (left.successRate !== right.successRate) {
    return right.successRate - left.successRate
  }

  return left.agent.localeCompare(right.agent)
}

export const rankLeaderboard = (
  entries: readonly LeaderboardSeed[],
): LeaderboardEntry[] =>
  [...entries]
    .sort(compareEntries)
    .map((entry, index) => ({
      ...entry,
      rank: index + 1,
    }))


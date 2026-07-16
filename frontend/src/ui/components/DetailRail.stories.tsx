import type { Meta, StoryObj } from '@storybook/react-vite'
import type { EventLog } from '../../domain/harbor'
import { getTranslator } from '../../i18n'
import { events, jobs, trialRows } from '../../mocks/demo'
import { DetailRail } from './DetailRail'

const t = getTranslator('en')

const longEvents: EventLog[] = [
  ...events,
  { time: '14:20:01', level: 'info', message: 'Container image layer hb__terminal-bench restored from cache' },
  { time: '14:20:18', level: 'info', message: 'Trial terminal-bench/git-rebase-conflict streamed stdout line 184' },
  { time: '14:21:04', level: 'warning', message: 'Verifier timeout multiplier applied to long-running shell assertion' },
  { time: '14:21:52', level: 'info', message: 'Artifact writer appended result metadata to job directory' },
  { time: '14:22:31', level: 'success', message: 'Checkpoint persisted for resumable Harbor job state' },
  { time: '14:23:16', level: 'info', message: 'Trial terminal-bench/sqlite-log-repair queued after concurrency slot released' },
  { time: '14:24:02', level: 'info', message: 'Log stream continued with a long diagnostic line that wraps inside the scroll window instead of widening the drawer' },
]

const meta = {
  title: 'Components/DetailRail',
  component: DetailRail,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof DetailRail>

export default meta
type Story = StoryObj<typeof meta>

export const JobDetail: Story = {
  args: {
    job: jobs[0],
    events,
    trials: trialRows.filter((row) => row.jobId === jobs[0].id),
    t,
    onJobAction: () => undefined,
    onCopyJob: () => undefined,
    onLeaderboardChange: () => undefined,
  },
}

export const RecoverableJobAction: Story = {
  args: {
    job: jobs.find((job) => job.status === 'failed' && job.canResume)!,
    events,
    trials: trialRows.filter((row) => row.jobId === jobs.find((job) => job.status === 'failed' && job.canResume)?.id),
    t,
    onJobAction: () => undefined,
    onCopyJob: () => undefined,
    onLeaderboardChange: () => undefined,
  },
}

export const FailedWithoutResumeArtifacts: Story = {
  args: {
    job: { ...jobs.find((job) => job.status === 'failed')!, canResume: false },
    events,
    trials: [],
    t,
    onJobAction: () => undefined,
    onCopyJob: () => undefined,
    onLeaderboardChange: () => undefined,
  },
}

export const ScrollableEventLog: Story = {
  args: {
    job: jobs[0],
    events: longEvents,
    trials: trialRows.filter((row) => row.jobId === jobs[0].id),
    t,
    onJobAction: () => undefined,
    onCopyJob: () => undefined,
    onLeaderboardChange: () => undefined,
  },
  decorators: [
    (StoryComponent) => (
      <main className="workspace single-page" style={{ maxWidth: 640 }}>
        <StoryComponent />
      </main>
    ),
  ],
}

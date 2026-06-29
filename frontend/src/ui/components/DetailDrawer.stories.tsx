import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { getTranslator } from '../../i18n'
import { events, jobs, trialRows } from '../../mocks/demo'
import { DetailDrawer } from './DetailDrawer'
import { DetailRail } from './DetailRail'

const t = getTranslator('en')

function DetailDrawerFixture() {
  const [open, setOpen] = useState(true)
  const job = jobs[0]

  return (
    <main className="workspace single-page">
      <section className="surface rail-card">
        <h1>Resizable Detail Drawer</h1>
        <div className="button-row tight">
          <button className="secondary-button" onClick={() => setOpen(true)}>
            Open drawer
          </button>
        </div>
      </section>
      <DetailDrawer label="Selected job" open={open} onClose={() => setOpen(false)}>
        <DetailRail job={job} events={events} trials={trialRows.filter((row) => row.jobId === job.id)} t={t} />
      </DetailDrawer>
    </main>
  )
}

const meta = {
  title: 'Components/DetailDrawer',
  component: DetailDrawerFixture,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof DetailDrawerFixture>

export default meta
type Story = StoryObj<typeof meta>

export const Resizable: Story = {}

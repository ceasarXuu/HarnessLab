import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { getTranslator } from '../../i18n'
import { Pagination } from './Pagination'

const t = getTranslator('en')

const meta = {
  title: 'Components/Pagination',
  component: Pagination,
} satisfies Meta<typeof Pagination>

export default meta
type Story = StoryObj<typeof meta>

function PaginationFixture() {
  const [page, setPage] = useState(1)
  return (
    <Pagination
      endItem={page * 20}
      page={page}
      startItem={(page - 1) * 20 + 1}
      t={t}
      totalItems={64}
      totalPages={4}
      onPage={setPage}
    />
  )
}

export const Default: Story = {
  args: {
    endItem: 20,
    page: 1,
    startItem: 1,
    t,
    totalItems: 64,
    totalPages: 4,
    onPage: () => undefined,
  },
  render: () => <PaginationFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByText('1-20 of 64')).toBeVisible()
    await userEvent.click(canvas.getByRole('button', { name: 'Next page' }))
    await expect(canvas.getByText('21-40 of 64')).toBeVisible()
  },
}

export const Empty: Story = {
  args: {
    endItem: 0,
    page: 1,
    startItem: 0,
    t,
    totalItems: 0,
    totalPages: 1,
    onPage: () => undefined,
  },
}

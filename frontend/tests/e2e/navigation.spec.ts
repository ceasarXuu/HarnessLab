import { expect, test } from '@playwright/test'

test('navigates the primary console views', async ({ page }) => {
  await page.goto('/')

  await expect(page.getByRole('heading', { name: 'Operations Dashboard' })).toBeVisible()

  await page.getByRole('link', { name: 'Agents' }).click()
  await expect(page.getByRole('heading', { name: 'Agent Fleet' })).toBeVisible()

  await page.getByRole('link', { name: 'Experiments' }).click()
  await expect(page.getByRole('heading', { name: 'Experiment Pipeline' })).toBeVisible()

  await page.getByRole('link', { name: 'Leaderboard' }).click()
  await expect(page.getByRole('heading', { name: 'Benchmark Leaderboard' })).toBeVisible()
})

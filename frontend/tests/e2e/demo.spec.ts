import { expect, test } from '@playwright/test'

test('renders primary Harbor WebUI demo surfaces', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('link', { name: /OrnnLab/ })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Jobs' })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'New Run' })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'System doctor' })).toBeVisible()
})

test('launch action creates a queued draft job', async ({ page }) => {
  await page.goto('/')
  await page.locator('#new-run').getByRole('button', { name: 'Run job' }).click()
  await expect(page.getByRole('button', { name: 'terminal-bench-draft' })).toBeVisible()
})

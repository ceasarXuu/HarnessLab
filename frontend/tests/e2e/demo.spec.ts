import { expect, test } from '@playwright/test'

test('renders primary Harbor WebUI demo surfaces', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('link', { name: /OrnnLab/ })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Job registry' })).toBeVisible()
  await expect(page.getByText('Selected job')).toBeVisible()
  await expect(page.getByRole('heading', { name: 'System doctor' })).toBeVisible()
})

test('launch action creates a queued draft job', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Run job' }).click()
  await expect(page.getByRole('heading', { name: 'New Run' })).toBeVisible()
  await page.locator('#new-run').getByRole('button', { name: 'Run job' }).click()
  await expect(page.getByRole('button', { name: 'terminal-bench-draft' })).toBeVisible()
})

test('navigates secondary demo pages and toggles preferences', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('link', { name: 'Tasks' }).click()
  await expect(page.getByRole('heading', { name: 'Task queue' })).toBeVisible()
  await page.getByRole('link', { name: 'Trials' }).click()
  await expect(page.getByRole('heading', { name: 'Trial matrix' })).toBeVisible()
  await page.getByRole('link', { name: 'System' }).click()
  await expect(page.getByRole('heading', { name: 'System health' })).toBeVisible()
  await page.getByLabel('Language').selectOption('zh')
  await expect(page.getByRole('heading', { name: '系统健康' })).toBeVisible()
  await page.getByRole('button', { name: '深色' }).click()
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
})

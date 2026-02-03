import { test, expect } from '@playwright/test';
import { _electron as electron } from 'playwright';
import path from 'path';

test('Start Server Flow', async () => {
  const appPath = path.join(__dirname, '..', '..', 'src-tauri', 'target', 'debug', 'app.exe');
  
  const electronApp = await electron.launch({
    executablePath: appPath,
  });

  const window = await electronApp.firstWindow();
  await window.waitForSelector('.app-container');

  // Find an instance and click it
  const instanceCard = window.locator('.instance-card').first();
  await instanceCard.click();

  // Click Start button
  await window.click('button:has-text("Start")');

  // Verify status change
  const statusBadge = window.locator('.status-badge');
  await expect(statusBadge).toContainText('Starting', { timeout: 15000 });
  await expect(statusBadge).toContainText('Running', { timeout: 60000 });

  // Verify console output
  const consoleLine = window.locator('.console-line');
  await expect(consoleLine.last()).toContainText('Done', { timeout: 60000 });

  await electronApp.close();
});

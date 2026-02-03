import { test, expect } from '@playwright/test';
import { _electron as electron } from 'playwright';
import path from 'path';

test('Create Instance Flow', async () => {
  // Path to your Tauri binary
  const appPath = path.join(__dirname, '..', '..', 'src-tauri', 'target', 'debug', 'app.exe');
  
  const electronApp = await electron.launch({
    executablePath: appPath,
  });

  const window = await electronApp.firstWindow();
  
  // Wait for the app to load
  await window.waitForSelector('.app-container', { timeout: 10000 });

  // Click on "Create Instance" button
  await window.click('text=Create Instance');

  // Fill in instance name
  await window.fill('input[placeholder="Instance Name"]', 'Test Instance');

  // Select version (assuming a custom Select component)
  await window.click('.select-trigger');
  await window.click('text=1.20.1');

  // Click Create
  await window.click('button:has-text("Create")');

  // Verify success toast
  const toast = window.locator('.toast-success');
  await expect(toast).toBeVisible();
  await expect(toast).toContainText('Instance created successfully');

  // Verify instance appears in the list
  const instanceCard = window.locator('.instance-card', { hasText: 'Test Instance' });
  await expect(instanceCard).toBeVisible();

  await electronApp.close();
});

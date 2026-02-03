import { test, expect } from '@playwright/test';
import { chromium } from 'playwright';
import path from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

test('Server Full Lifecycle Flow', async () => {
  test.setTimeout(240000); // Increased timeout for the full flow
  
  const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

  // Path to your Tauri binary
  const appPath = path.join(__dirname, '..', '..', 'src-tauri', 'target', 'debug', 'app.exe');
  
  // Start the Tauri app with a remote debugging port
  const tauriProcess = spawn(appPath, [], {
    stdio: 'ignore',
    detached: true,
    env: {
      ...process.env,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9222',
    }
  });

  // Give it a moment to start
  await new Promise(resolve => setTimeout(resolve, 15000));

  let browser;
  try {
    // Connect to the Tauri app via CDP
    console.log('Connecting to CDP...');
    browser = await chromium.connectOverCDP('http://localhost:9222');
    console.log('Connected to CDP');
    const defaultContext = browser.contexts()[0];
    const page = defaultContext.pages()[0] || await defaultContext.newPage();

    // Force load the dev server URL to ensure we're on the right page
    console.log('Navigating to http://localhost:3000...');
    await page.goto('http://localhost:3000');
    console.log('Navigated to http://localhost:3000');

    // Wait for the app to load
    console.log('Waiting for app to load...');
    await page.waitForSelector('text=Create New Instance', { timeout: 30000 });
    console.log('App loaded');

    // --- STEP 1: CREATE INSTANCE ---
    console.log('--- Step 1: Creating Instance ---');
    await delay(1000);
    console.log('Clicking "Create New Instance"...');
    await page.click('text=Create New Instance');
    await delay(500);

    // Select Server Software (e.g., Vanilla)
    console.log('Selecting Vanilla software...');
    await page.click('span:text-is("Vanilla")');
    await delay(500);

    // Fill in instance name
    console.log('Filling instance name...');
    const instanceName = 'Test Instance ' + Date.now(); // Unique name to avoid conflicts
    await page.fill('input[placeholder="Enter instance name..."]', instanceName);
    await delay(500);

    // Select version
    console.log('Selecting version 1.20.1...');
    const versionRow = page.locator('tr').filter({ hasText: '1.20.1' }).first();
    await versionRow.waitFor({ state: 'visible', timeout: 30000 });
    await versionRow.click();
    console.log('Version 1.20.1 selected');
    await delay(500);

    // Click Create
    console.log('Clicking Create Instance...');
    await page.click('button:has-text("Create Instance")');

    // Verify modal closes
    console.log('Waiting for modal to close...');
    await expect(page.locator('text=Ready to install Minecraft')).not.toBeVisible({ timeout: 15000 });
    console.log('Modal closed');
    await delay(1000);

    // Verify instance appears in the list (in Sidebar)
    console.log(`Verifying instance "${instanceName}" in sidebar...`);
    const instanceCard = page.locator('button').filter({ hasText: instanceName }).first();
    await expect(instanceCard).toBeVisible({ timeout: 15000 });
    console.log('Instance created and verified');
    await delay(1000);

    // --- STEP 2: START SERVER ---
    console.log('--- Step 2: Starting Server ---');
    console.log('Selecting instance in sidebar...');
    await instanceCard.click();
    console.log('Instance selected');
    await delay(1000);

    // Click Start button
    console.log('Clicking "Start Server"...');
    await page.click('button:has-text("Start Server")');

    // Verify status change
    console.log('Waiting for "Starting..." status...');
    await expect(page.locator('text=Starting...').first()).toBeVisible({ timeout: 15000 });
    
    console.log('Waiting for "Running" status...');
    await expect(page.locator('text=Running').first()).toBeVisible({ timeout: 90000 });
    console.log('Status: Running');

    // Verify console output
    console.log('Verifying console output says "Done"...');
    await page.click('text=Console');
    await expect(page.locator('text=Done')).toBeVisible({ timeout: 90000 });
    console.log('Server successfully turned on');

    // --- STEP 3: STOP SERVER ---
    console.log('--- Step 3: Stopping Server ---');
    console.log('Clicking "Stop Server"...');
    // The stop button is usually in the Header when a server is running
    const stopButton = page.locator('button:has-text("Stop Server")').first();
    await stopButton.click();

    console.log('Waiting for "Stopping..." status...');
    await expect(page.locator('text=Stopping...').first()).toBeVisible({ timeout: 15000 });

    console.log('Waiting for "Stopped" status (Offline)...');
    // In GlobalDashboard it says "Offline", in Header it might say something else or just show Start button
    // Let's check for the Start Server button to reappear or the status text
    await expect(page.locator('button:has-text("Start Server")')).toBeVisible({ timeout: 60000 });
    console.log('Server stopped successfully');

    // --- STEP 4: DELETE INSTANCE ---
    console.log('--- Step 4: Deleting Instance ---');
    // Hover to reveal the settings button in sidebar
    await instanceCard.hover();

    // Click the settings (gear) button for this instance
    console.log('Clicking settings button...');
    const settingsButton = instanceCard.locator('button').filter({ has: page.locator('svg.lucide-settings') }).first();
    await settingsButton.click();
    console.log('Settings dropdown opened');

    // Click "Delete Instance" in the dropdown
    console.log('Clicking "Delete Instance"...');
    const deleteButton = page.locator('button').filter({ hasText: 'Delete Instance' }).first();
    await deleteButton.waitFor({ state: 'visible', timeout: 5000 });
    await deleteButton.click();

    // Click "Permanently Delete" in the confirmation
    console.log('Clicking "Permanently Delete"...');
    const confirmDeleteButton = page.locator('button').filter({ hasText: 'Permanently Delete' }).first();
    await confirmDeleteButton.waitFor({ state: 'visible', timeout: 5000 });
    await confirmDeleteButton.click();

    // Verify instance is gone from the sidebar
    console.log('Verifying instance is deleted...');
    await expect(page.locator('button').filter({ hasText: instanceName })).toHaveCount(0, { timeout: 15000 });
    console.log('Instance deletion verified');

  } finally {
    console.log('Cleaning up...');
    if (browser) await browser.close();
    tauriProcess.kill();
  }
});

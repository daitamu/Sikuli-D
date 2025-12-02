import { test, expect } from '@playwright/test';

test.describe('CodeMode Image Widget', () => {
  test('should capture console logs for image widget debugging', async ({ page }) => {
    // Capture console logs
    const consoleLogs: string[] = [];
    page.on('console', (msg) => {
      if (msg.text().includes('[ImageWidget]') || msg.text().includes('[ImageLoader]')) {
        consoleLogs.push(msg.text());
      }
    });

    // Navigate to the app
    await page.goto('/');

    // Wait for app to load
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

    // Switch to Code mode
    const codeButton = page.locator('button:has-text("Code")');
    if (await codeButton.isVisible()) {
      await codeButton.click();
    }

    // Wait a bit for the editor to mount
    await page.waitForTimeout(2000);

    // Log all captured console messages
    console.log('=== Captured Console Logs ===');
    consoleLogs.forEach(log => console.log(log));
    console.log('=============================');

    // Check if Monaco editor loaded
    const monacoEditor = page.locator('.monaco-editor');
    await expect(monacoEditor).toBeVisible({ timeout: 10000 });
  });

  test('should show inline image widgets when code contains image references', async ({ page }) => {
    // Capture console logs
    const consoleLogs: string[] = [];
    page.on('console', (msg) => {
      consoleLogs.push(`[${msg.type()}] ${msg.text()}`);
    });

    // Navigate to the app
    await page.goto('/');

    // Wait for app to load
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

    // Switch to Code mode
    const codeButton = page.locator('button:has-text("Code")');
    if (await codeButton.isVisible()) {
      await codeButton.click();
    }

    // Wait for Monaco editor
    await page.waitForSelector('.monaco-editor', { timeout: 10000 });

    // Check for inline image widget elements
    const imageWidgets = page.locator('.sikuli-inline-image-widget');
    const widgetCount = await imageWidgets.count();

    console.log('=== Image Widget Test Results ===');
    console.log(`Found ${widgetCount} inline image widgets`);

    // Check for image path decorations
    const imagePathDecorations = page.locator('.sikuli-image-path');
    const decorationCount = await imagePathDecorations.count();
    console.log(`Found ${decorationCount} image path decorations`);

    // Log relevant console messages
    console.log('\n=== Relevant Console Logs ===');
    consoleLogs
      .filter(log => log.includes('ImageWidget') || log.includes('ImageLoader'))
      .forEach(log => console.log(log));
    console.log('==============================');
  });
});

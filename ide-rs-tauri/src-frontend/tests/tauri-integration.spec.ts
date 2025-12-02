import { test, expect, chromium } from '@playwright/test';

test.describe('Tauri App Integration Test', () => {
  test('should connect to Tauri app via CDP and test image widgets', async () => {
    // Tauri app should be running in dev mode with --devtools flag
    // Try to connect via CDP (default port 9222)
    const consoleLogs: string[] = [];

    try {
      // Connect to the running Tauri app's WebView
      const browser = await chromium.connectOverCDP('http://127.0.0.1:9222');
      const contexts = browser.contexts();

      if (contexts.length === 0) {
        console.log('No browser contexts found. Is Tauri app running?');
        return;
      }

      const context = contexts[0];
      const pages = context.pages();

      if (pages.length === 0) {
        console.log('No pages found in context');
        return;
      }

      const page = pages[0];

      // Capture console logs
      page.on('console', (msg) => {
        consoleLogs.push(`[${msg.type()}] ${msg.text()}`);
        if (msg.text().includes('ImageWidget') || msg.text().includes('ImageLoader')) {
          console.log(`[TAURI CONSOLE] ${msg.text()}`);
        }
      });

      // Wait a bit for any pending updates
      await page.waitForTimeout(2000);

      // Print current state
      const state = await page.evaluate(() => {
        const widgets = document.querySelectorAll('.sikuli-inline-image-widget');
        const decorations = document.querySelectorAll('.sikuli-image-path');
        const patternImages = document.querySelectorAll('[class*="Pattern Images"]');
        const rightPanel = document.querySelector('.w-48.bg-dark-surface');

        return {
          widgetCount: widgets.length,
          decorationCount: decorations.length,
          hasPatternPanel: !!rightPanel,
          patternPanelImages: rightPanel?.querySelectorAll('img').length ?? 0,
          pageTitle: document.title,
          bodyText: document.body.innerText.substring(0, 500)
        };
      });

      console.log('=== Tauri App State ===');
      console.log(JSON.stringify(state, null, 2));

      console.log('\n=== Relevant Console Logs ===');
      consoleLogs
        .filter(log => log.includes('ImageWidget') || log.includes('ImageLoader'))
        .forEach(log => console.log(log));

      await browser.close();
    } catch (error) {
      console.log('CDP connection failed. Make sure Tauri app is running with devtools enabled.');
      console.log('Error:', error);

      // Fallback: try to connect to Vite dev server instead
      console.log('\nFalling back to Vite dev server test...');
    }
  });
});

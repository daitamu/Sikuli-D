import { test, chromium } from '@playwright/test';

test('現在起動中のIDEの状態を確認', async () => {
  // Vite dev serverに接続
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  const consoleLogs: string[] = [];
  page.on('console', (msg) => {
    const text = msg.text();
    consoleLogs.push(`[${msg.type()}] ${text}`);
  });

  await page.goto('http://localhost:5173');
  await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

  // Codeモードに切り替え
  const codeButton = page.locator('button:has-text("Code")');
  if (await codeButton.isVisible()) {
    await codeButton.click();
  }

  await page.waitForSelector('.monaco-editor', { timeout: 10000 });
  await page.waitForTimeout(2000);

  // 現在の状態を取得
  const state = await page.evaluate(() => {
    const monaco = (window as any).monaco;
    const editor = monaco?.editor?.getEditors?.()?.[0];

    return {
      hasEditor: !!editor,
      editorValue: editor?.getValue()?.substring(0, 200) || 'No editor',
      widgetCount: document.querySelectorAll('.sikuli-inline-image-widget').length,
      decorationCount: document.querySelectorAll('.sikuli-image-path').length,
      patternPanelImages: document.querySelectorAll('.w-48.bg-dark-surface img')?.length || 0,
    };
  });

  console.log('\n=== 現在のIDE状態 ===');
  console.log(JSON.stringify(state, null, 2));

  console.log('\n=== ImageWidget/ImageLoader ログ ===');
  consoleLogs
    .filter(log => log.includes('[ImageWidget]') || log.includes('[ImageLoader]'))
    .forEach(log => console.log(log));

  await page.screenshot({ path: 'test-results/current-ide-state.png' });
  console.log('\nスクリーンショット保存: test-results/current-ide-state.png');

  await browser.close();
});

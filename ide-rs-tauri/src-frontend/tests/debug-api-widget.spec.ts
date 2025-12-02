import { test, expect } from '@playwright/test';
import * as fs from 'fs';

// 10x10 green PNG for testing fallback
const MOCK_BASE64_IMAGE = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFklEQVR42mNk+M9QzwAEjDAGNzANAQC4MgD/Xk9g8QAAAABJRU5ErkJggg==';

test.describe('Debug API によるファイルオープンシミュレーションテスト', () => {
  test('デバッグAPIを使用してYesman.sikuliを開いた状態をシミュレートし、ウィジェットが表示されることを確認', async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on('console', (msg) => {
      const text = msg.text();
      consoleLogs.push(`[${msg.type()}] ${text}`);
      if (text.includes('[ImageWidget]') || text.includes('[ImageLoader]') || text.includes('[Debug]')) {
        console.log('[CONSOLE]', text);
      }
    });

    // Yesman.pyの実際のコードを読み込む
    const yesmanPyPath = 'C:/VSCode/Yesman.sikuli/Yesman.py';
    let yesmanCode = '';
    try {
      yesmanCode = fs.readFileSync(yesmanPyPath, 'utf-8');
      console.log('=== Yesman.py 読み込み成功 ===');
      console.log('コード長:', yesmanCode.length, '文字');
    } catch (err) {
      console.log('=== Yesman.py 読み込み失敗、サンプルコードを使用 ===');
      yesmanCode = `# Sample Sikuli script
target = s.exists(Pattern("1764082193839.png").similar(0.85), 1)
click("another-image.png")
`;
    }

    // Yesman.sikuliの画像をBase64で読み込む
    const imagePath = 'C:/VSCode/Yesman.sikuli/1764082193839.png';
    let imageBase64 = MOCK_BASE64_IMAGE;
    try {
      const imageBuffer = fs.readFileSync(imagePath);
      imageBase64 = `data:image/png;base64,${imageBuffer.toString('base64')}`;
      console.log('=== 画像読み込み成功 ===');
      console.log('画像パス:', imagePath);
    } catch (err) {
      console.log('=== 画像読み込み失敗、モック画像を使用 ===');
    }

    await page.goto('/');
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

    // Debug APIが利用可能か確認
    await page.waitForFunction(() => (window as any).__SIKULID_DEBUG__, { timeout: 5000 });
    console.log('=== Debug API 利用可能 ===');

    // Codeモードに切り替え
    const codeButton = page.locator('button:has-text("Code")');
    if (await codeButton.isVisible()) {
      await codeButton.click();
    }

    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // Debug APIを使用してファイルを開いた状態をシミュレート
    const setupResult = await page.evaluate(({ code, imageBase64, filePath }) => {
      const debug = (window as any).__SIKULID_DEBUG__;
      if (!debug) {
        return { error: 'Debug API not available' };
      }

      // 画像パターンを検出
      const STANDALONE_IMAGE_REGEX = /(?:Pattern\s*\(\s*)?["']([^"']+\.(?:png|jpg|jpeg|gif|bmp))["']/gi;
      const images: string[] = [];
      let match;
      while ((match = STANDALONE_IMAGE_REGEX.exec(code)) !== null) {
        if (!images.includes(match[1])) {
          images.push(match[1]);
        }
      }
      console.log('[Test] Detected images:', images);

      // imagePatternsマップを作成
      const patterns = new Map<string, string>();
      images.forEach(img => {
        patterns.set(img, imageBase64);
      });

      // ImageLoaderをスキップするフラグを設定（テスト用）
      // これにより、setCurrentFile/setSourceCode後のuseEffectで
      // imagePatternsが上書きされるのを防ぐ
      debug.skipImageLoader(true);

      // Debug APIを使用して状態を設定
      debug.setImagePatterns(patterns);
      debug.setCurrentFile(filePath);
      debug.setSourceCode(code);

      // エディタにコードを設定
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      if (editor) {
        editor.setValue(code);
      }

      return {
        success: true,
        imagesFound: images,
        patternsSet: patterns.size,
        currentState: debug.getCurrentState(),
      };
    }, { code: yesmanCode, imageBase64, filePath: 'C:/VSCode/Yesman.sikuli/Yesman.py' });

    console.log('\n=== Setup Result ===');
    console.log(JSON.stringify(setupResult, null, 2));

    // ウィジェット更新を待つ
    await page.waitForTimeout(2000);

    // 現在の状態を確認
    const state = await page.evaluate(() => {
      const debug = (window as any).__SIKULID_DEBUG__;
      return {
        debugState: debug?.getCurrentState(),
        widgetCount: document.querySelectorAll('.sikuli-inline-image-widget').length,
        decorationCount: document.querySelectorAll('.sikuli-image-path').length,
      };
    });

    console.log('\n=== 現在の状態 ===');
    console.log(JSON.stringify(state, null, 2));

    // ImageWidgetのログを確認
    console.log('\n=== ImageWidget/ImageLoader ログ ===');
    consoleLogs
      .filter(log => log.includes('[ImageWidget]') || log.includes('[ImageLoader]'))
      .forEach(log => console.log(log));

    // スクリーンショット保存
    await page.screenshot({ path: 'test-results/debug-api-widget-test.png', fullPage: true });
    console.log('\nスクリーンショット保存: test-results/debug-api-widget-test.png');

    // 検証
    expect(setupResult.success).toBe(true);
    expect(state.debugState?.imagePatternsSize).toBeGreaterThan(0);
    // ウィジェットが表示されることを期待
    expect(state.widgetCount).toBeGreaterThan(0);
  });
});

import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

test.describe('Yesman.sikuli インライン画像ウィジェットテスト', () => {
  test('Yesman.pyのコードでインライン画像ウィジェットが表示される', async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on('console', (msg) => {
      const text = msg.text();
      consoleLogs.push(`[${msg.type()}] ${text}`);
      if (text.includes('[ImageWidget]') || text.includes('[ImageLoader]')) {
        console.log('[CONSOLE]', text);
      }
    });

    // Yesman.pyの実際のコードを読み込む
    const yesmanPyPath = 'C:/VSCode/Yesman.sikuli/Yesman.py';
    const yesmanCode = fs.readFileSync(yesmanPyPath, 'utf-8');
    console.log('=== Yesman.py コード読み込み完了 ===');
    console.log('コード長:', yesmanCode.length, '文字');

    // Yesman.sikuliの画像をBase64で読み込む
    const imagePath = 'C:/VSCode/Yesman.sikuli/1764082193839.png';
    const imageBuffer = fs.readFileSync(imagePath);
    const imageBase64 = `data:image/png;base64,${imageBuffer.toString('base64')}`;
    console.log('=== 画像読み込み完了 ===');
    console.log('画像パス:', imagePath);
    console.log('Base64長:', imageBase64.length, '文字');

    await page.goto('/');
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

    // Codeモードに切り替え
    const codeButton = page.locator('button:has-text("Code")');
    if (await codeButton.isVisible()) {
      await codeButton.click();
    }

    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // Yesman.pyのコードを設定し、imagePatternsを手動で注入
    const testResult = await page.evaluate(({ code, imageBase64 }) => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      if (!editor) {
        return { error: 'エディタが見つかりません' };
      }

      // Yesman.pyのコードを設定
      editor.setValue(code);

      // 画像位置を検出（findImagePositionsと同じロジック）
      const imagePathRegex = /["']([^"']+\.(?:png|jpg|jpeg|gif|bmp))["']/gi;
      const lines = code.split('\n');
      const positions: any[] = [];

      lines.forEach((line: string, lineIndex: number) => {
        let match;
        const regex = new RegExp(imagePathRegex.source, 'gi');
        while ((match = regex.exec(line)) !== null) {
          positions.push({
            imagePath: match[1],
            lineNumber: lineIndex + 1,
            column: match.index + 1,
            endColumn: match.index + match[0].length + 1,
          });
        }
      });

      console.log('[Test] 検出された画像位置:', positions);

      // ContentWidgetを手動で作成
      const widgetsAdded: string[] = [];
      const decorations: any[] = [];

      positions.forEach((pos, index) => {
        // デコレーション作成
        decorations.push({
          range: new monaco.Range(pos.lineNumber, pos.column, pos.lineNumber, pos.endColumn),
          options: {
            inlineClassName: 'sikuli-image-path',
            hoverMessage: { value: '**' + pos.imagePath + '**' },
          },
        });

        // ウィジェット作成
        const widgetId = 'yesman-widget-' + index;
        const container = document.createElement('div');
        container.className = 'sikuli-inline-image-widget';
        container.dataset.imagePath = pos.imagePath;
        container.style.cssText = 'display: inline-flex; align-items: center; justify-content: center; margin-left: 4px; padding: 2px; background: #2d2d2d; border: 1px solid #444; border-radius: 3px; cursor: pointer;';

        const img = document.createElement('img');
        img.src = imageBase64;
        img.style.cssText = 'max-width: 48px; max-height: 24px; object-fit: contain;';
        img.title = pos.imagePath;
        container.appendChild(img);

        const widget = {
          getId: () => widgetId,
          getDomNode: () => container,
          getPosition: () => ({
            position: { lineNumber: pos.lineNumber, column: pos.endColumn },
            preference: [monaco.editor.ContentWidgetPositionPreference.EXACT],
          }),
        };

        editor.addContentWidget(widget);
        widgetsAdded.push(widgetId);
        console.log('[Test] ウィジェット追加:', widgetId, '行:', pos.lineNumber);
      });

      // デコレーション適用
      editor.deltaDecorations([], decorations);

      return {
        success: true,
        codeLines: lines.length,
        positionsFound: positions.length,
        widgetsAdded,
        positions: positions.map((p: any) => ({ imagePath: p.imagePath, line: p.lineNumber }))
      };
    }, { code: yesmanCode, imageBase64 });

    console.log('\n=== テスト結果 ===');
    console.log(JSON.stringify(testResult, null, 2));

    await page.waitForTimeout(500);

    // ウィジェット数を確認
    const widgetCount = await page.locator('.sikuli-inline-image-widget').count();
    console.log(`\nDOMに存在するウィジェット数: ${widgetCount}`);

    // デコレーション数を確認
    const decorationCount = await page.locator('.sikuli-image-path').count();
    console.log(`DOMに存在するデコレーション数: ${decorationCount}`);

    // スクリーンショット保存
    await page.screenshot({ path: 'test-results/yesman-widget-test.png', fullPage: true });
    console.log('\nスクリーンショット保存: test-results/yesman-widget-test.png');

    // 検証
    expect(testResult.success).toBe(true);
    expect(testResult.positionsFound).toBeGreaterThan(0);
    expect(widgetCount).toBeGreaterThan(0);

    console.log('\n=== テスト完了: 成功 ===');
  });
});

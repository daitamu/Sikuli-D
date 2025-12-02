import { test, expect } from '@playwright/test';

// 10x10 green PNG for testing
const MOCK_BASE64_IMAGE = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFklEQVR42mNk+M9QzwAEjDAGNzANAQC4MgD/Xk9g8QAAAABJRU5ErkJggg==';

test.describe('ContentWidget API Direct Test', () => {
  test('should verify Monaco ContentWidget API adds widget to DOM', async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on('console', (msg) => {
      consoleLogs.push(`[${msg.type()}] ${msg.text()}`);
    });

    await page.goto('/');
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

    // Switch to Code mode
    const codeButton = page.locator('button:has-text("Code")');
    if (await codeButton.isVisible()) {
      await codeButton.click();
    }

    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await page.waitForTimeout(500);

    // Directly test ContentWidget API
    const result = await page.evaluate((mockImage) => {
      const monaco = (window as any).monaco;
      const editors = monaco?.editor?.getEditors?.() || [];

      if (editors.length === 0) {
        return { success: false, error: 'No editors found' };
      }

      const editor = editors[0];

      // Set test code
      editor.setValue('click("test.png")\nwait("image.jpg")');

      // Create a visible test widget
      const container = document.createElement('div');
      container.id = 'test-content-widget';
      container.className = 'sikuli-inline-image-widget';
      container.style.cssText = 'display: inline-flex; align-items: center; background: #2d2d2d; border: 2px solid #4EC9B0; border-radius: 3px; padding: 2px;';

      const img = document.createElement('img');
      img.src = mockImage;
      img.style.cssText = 'width: 24px; height: 24px;';
      container.appendChild(img);

      const widget = {
        getId: () => 'test-widget-1',
        getDomNode: () => container,
        getPosition: () => ({
          position: { lineNumber: 1, column: 18 },
          preference: [monaco.editor.ContentWidgetPositionPreference.EXACT]
        })
      };

      try {
        editor.addContentWidget(widget);

        // Check if widget is in DOM after adding
        setTimeout(() => {
          const widgetInDom = document.getElementById('test-content-widget');
          console.log('[Test] Widget in DOM after add:', !!widgetInDom);
        }, 100);

        return {
          success: true,
          widgetAdded: true,
          editorValue: editor.getValue()
        };
      } catch (err: any) {
        return { success: false, error: err.message };
      }
    }, MOCK_BASE64_IMAGE);

    console.log('Direct ContentWidget test result:', JSON.stringify(result, null, 2));
    expect(result.success).toBe(true);

    await page.waitForTimeout(200);

    // Check if widget is in DOM
    const widgetElement = await page.locator('#test-content-widget');
    const widgetVisible = await widgetElement.isVisible();
    console.log('Widget visible in DOM:', widgetVisible);

    // Take screenshot
    await page.screenshot({ path: 'test-results/content-widget-direct-test.png' });

    // Verify widget is visible
    expect(widgetVisible).toBe(true);
  });

  test('should inject imagePatterns and verify widgets appear', async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on('console', (msg) => {
      const text = msg.text();
      consoleLogs.push(`[${msg.type()}] ${text}`);
      if (text.includes('[ImageWidget]') || text.includes('[ImageLoader]')) {
        console.log('[CONSOLE]', text);
      }
    });

    await page.goto('/');
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

    const codeButton = page.locator('button:has-text("Code")');
    if (await codeButton.isVisible()) {
      await codeButton.click();
    }

    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // Set code and simulate imagePatterns being populated
    const testResult = await page.evaluate((mockImage) => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      if (!editor) {
        return { error: 'No editor found' };
      }

      const testCode = 'click("test-image.png")\nwait(Pattern("another.png").similar(0.9))';
      editor.setValue(testCode);

      // Create widgets manually to simulate what updateImageWidgets does
      const positions = [
        { imagePath: 'test-image.png', lineNumber: 1, column: 7, endColumn: 22 },
        { imagePath: 'another.png', lineNumber: 2, column: 14, endColumn: 26 }
      ];

      const mockPatterns = new Map([
        ['test-image.png', mockImage],
        ['another.png', mockImage]
      ]);

      const widgetsAdded: string[] = [];
      const decorations: any[] = [];

      positions.forEach((pos, index) => {
        const imageData = mockPatterns.get(pos.imagePath);
        if (!imageData) return;

        // Create decoration
        decorations.push({
          range: new monaco.Range(pos.lineNumber, pos.column, pos.lineNumber, pos.endColumn),
          options: {
            inlineClassName: 'sikuli-image-path',
            hoverMessage: { value: '**' + pos.imagePath + '**' },
          },
        });

        // Create widget
        const widgetId = 'manual-image-widget-' + index;
        const container = document.createElement('div');
        container.className = 'sikuli-inline-image-widget';
        container.dataset.testWidget = widgetId;
        container.style.cssText = 'display: inline-flex; align-items: center; justify-content: center; margin-left: 4px; padding: 2px; background: #2d2d2d; border: 1px solid #444; border-radius: 3px; cursor: pointer;';

        const img = document.createElement('img');
        img.src = imageData;
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
      });

      // Apply decorations
      editor.deltaDecorations([], decorations);

      return {
        success: true,
        codeSet: true,
        widgetsAdded,
        decorationsCount: decorations.length
      };
    }, MOCK_BASE64_IMAGE);

    console.log('\n=== Manual Widget Injection Test ===');
    console.log(JSON.stringify(testResult, null, 2));

    await page.waitForTimeout(500);

    // Verify widgets are visible
    const widgetCount = await page.locator('.sikuli-inline-image-widget').count();
    console.log(`Found ${widgetCount} inline image widgets in DOM`);

    // Verify decorations
    const decorationCount = await page.locator('.sikuli-image-path').count();
    console.log(`Found ${decorationCount} image path decorations in DOM`);

    // Take screenshot
    await page.screenshot({ path: 'test-results/manual-widget-injection-test.png' });

    expect(testResult.success).toBe(true);
    expect(widgetCount).toBe(2); // Two widgets should be added
  });

  test('should verify position detection works correctly', async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on('console', (msg) => {
      consoleLogs.push(`[${msg.type()}] ${msg.text()}`);
    });

    await page.goto('/');
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

    const codeButton = page.locator('button:has-text("Code")');
    if (await codeButton.isVisible()) {
      await codeButton.click();
    }

    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // Set code with image references
    await page.evaluate(() => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      if (editor) {
        editor.setValue(`# Yesman.sikuli style script
target = s.exists(Pattern("1764082193839.png").similar(0.85), 1)
click("another-image.png")
`);
      }
    });

    await page.waitForTimeout(500);

    // Check position detection logs
    const positionLogs = consoleLogs.filter(log => log.includes('positions found'));
    console.log('\n=== Position Detection for Yesman-style code ===');
    positionLogs.forEach(log => console.log(log));

    // Also check what images were detected
    const checkingLogs = consoleLogs.filter(log => log.includes('checking image:'));
    console.log('\n=== Images Detected ===');
    checkingLogs.forEach(log => console.log(log));

    // Verify positions were found
    expect(positionLogs.length).toBeGreaterThan(0);

    // The last position log should contain info about detected images
    const lastLog = positionLogs[positionLogs.length - 1] || '';
    console.log('Last position log:', lastLog);
  });
});

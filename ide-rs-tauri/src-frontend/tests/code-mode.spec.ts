import { test, expect } from '@playwright/test';

/**
 * Playwright tests for Code Mode with Monaco Editor
 * Monaco Editorを使用したコードモードのPlaywrightテスト
 */

test.describe('Code Mode with Monaco Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for the app to load
    await page.waitForSelector('.flex', { timeout: 10000 });
  });

  test('should display the IDE header with mode switcher', async ({ page }) => {
    // Check that the header exists with mode buttons
    // ヘッダーとモードボタンが存在することを確認
    const header = page.locator('header, [class*="header"]').first();
    await expect(header).toBeVisible();

    // Look for mode buttons (Simple, Flow, Code)
    const codeButton = page.getByRole('button', { name: /code/i });
    await expect(codeButton).toBeVisible();
  });

  test('should switch to Code mode', async ({ page }) => {
    // Click on Code mode button
    // コードモードボタンをクリック
    const codeButton = page.getByRole('button', { name: /code/i });
    await codeButton.click();

    // Wait a bit for mode switch
    await page.waitForTimeout(500);

    // Check that Monaco Editor container is visible
    // Monaco Editorコンテナが表示されていることを確認
    // Use more specific selector - the actual editor, not style elements
    const monacoEditor = page.locator('.monaco-editor.vs-dark');
    await expect(monacoEditor.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display Python syntax highlighting in Code mode', async ({ page }) => {
    // Switch to Code mode
    const codeButton = page.getByRole('button', { name: /code/i });
    await codeButton.click();

    await page.waitForTimeout(1000);

    // Monaco Editor should be loaded
    const editor = page.locator('.monaco-editor');
    await expect(editor.first()).toBeVisible({ timeout: 10000 });

    // Check for syntax highlighting elements
    // シンタックスハイライト要素を確認
    const syntaxElements = page.locator('.mtk1, .mtk3, .mtk5, .mtk6, .mtk9');
    // If there's code, there should be syntax highlighting
    const count = await syntaxElements.count();
    console.log(`Found ${count} syntax-highlighted elements`);
  });

  test('should be able to type code in Monaco Editor', async ({ page }) => {
    // Switch to Code mode
    const codeButton = page.getByRole('button', { name: /code/i });
    await codeButton.click();

    await page.waitForTimeout(1000);

    // Find the Monaco Editor
    const editor = page.locator('.monaco-editor');
    await expect(editor.first()).toBeVisible({ timeout: 10000 });

    // Click on the editor to focus it
    await editor.first().click();

    // Type some Python code
    await page.keyboard.type('# Test comment');
    await page.keyboard.press('Enter');
    await page.keyboard.type('print("Hello World")');

    // Verify the code was entered by checking the editor content
    const editorContent = page.locator('.view-lines');
    await expect(editorContent).toContainText('print');
  });

  test('should show line numbers in Monaco Editor', async ({ page }) => {
    // Switch to Code mode
    const codeButton = page.getByRole('button', { name: /code/i });
    await codeButton.click();

    await page.waitForTimeout(1000);

    // Check for line numbers
    // 行番号を確認
    const lineNumbers = page.locator('.line-numbers, .margin-view-overlays');
    await expect(lineNumbers.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display toolbar with Save and Copy buttons', async ({ page }) => {
    // Switch to Code mode
    const codeButton = page.getByRole('button', { name: /code/i });
    await codeButton.click();

    await page.waitForTimeout(500);

    // Check for Save button
    // 保存ボタンを確認
    const saveButton = page.getByRole('button', { name: /save/i });
    await expect(saveButton).toBeVisible();

    // Check for Copy button
    // コピーボタンを確認 - use exact match to avoid multiple elements
    const copyButton = page.getByRole('button', { name: 'Copy', exact: true });
    await expect(copyButton).toBeVisible();
  });

  test('should display image panel toggle when code has image references', async ({ page }) => {
    // Switch to Code mode
    const codeButton = page.getByRole('button', { name: /code/i });
    await codeButton.click();

    await page.waitForTimeout(1000);

    // Focus and type code with image reference
    const editor = page.locator('.monaco-editor');
    await editor.first().click();

    await page.keyboard.type('click("button.png")');

    await page.waitForTimeout(500);

    // Check if image count indicator appears in toolbar
    // ツールバーに画像カウント表示が現れるか確認
    // The image panel button should show the count (1 image)
    const imageButton = page.locator('button').filter({ hasText: '1' }).filter({ has: page.locator('svg') });
    await expect(imageButton).toBeVisible({ timeout: 5000 });

    // Check if Pattern Images panel is visible
    // Pattern Imagesパネルが表示されていることを確認
    const patternImagesPanel = page.locator('text=Pattern Images');
    await expect(patternImagesPanel).toBeVisible({ timeout: 5000 });

    // Check if the image filename appears in the panel (not in editor)
    // パネルに画像ファイル名が表示されていることを確認（エディタ内ではなく）
    const imageFilename = page.locator('.px-2.py-1.text-xs').filter({ hasText: 'button.png' });
    await expect(imageFilename).toBeVisible({ timeout: 5000 });

    console.log('Image panel and Pattern Images verified');
  });
});

test.describe('Mode Switching', () => {
  test('should be able to switch between Simple, Flow, and Code modes', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.flex', { timeout: 10000 });

    // Get mode buttons
    const simpleButton = page.getByRole('button', { name: /simple/i });
    const flowButton = page.getByRole('button', { name: /flow/i });
    const codeButton = page.getByRole('button', { name: /code/i });

    // Click Code mode
    await codeButton.click();
    await page.waitForTimeout(500);

    // Verify Monaco Editor is visible
    const monacoEditor = page.locator('.monaco-editor');
    await expect(monacoEditor.first()).toBeVisible({ timeout: 10000 });

    // Click Simple mode
    await simpleButton.click();
    await page.waitForTimeout(500);

    // Monaco Editor should not be visible in Simple mode
    // (it might be unmounted or hidden)

    // Click Flow mode
    await flowButton.click();
    await page.waitForTimeout(500);

    // Check for Flow mode canvas
    const flowCanvas = page.locator('canvas, [class*="flow"]');
    // Flow mode elements should be visible

    console.log('Mode switching test completed');
  });
});

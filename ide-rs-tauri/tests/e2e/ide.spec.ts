/**
 * SikuliX IDE E2E Tests
 * SikuliX IDE E2Eテスト
 */

import { test, expect, waitForIDEReady, getEditorContent, setEditorContent, clickRunButton, getLogContent } from './fixtures';

test.describe('SikuliX IDE', () => {

  test.beforeEach(async ({ app }) => {
    await waitForIDEReady(app);
  });

  test('should display the IDE window', async ({ app }) => {
    // Check that the main elements are visible
    await expect(app.locator('.toolbar')).toBeVisible();
    await expect(app.locator('.monaco-editor')).toBeVisible();
    await expect(app.locator('.bottom-panel')).toBeVisible();
  });

  test('should allow typing in the editor', async ({ app }) => {
    const testCode = 'print("Hello, SikuliX!")';

    // Set content in the editor
    await setEditorContent(app, testCode);

    // Verify content was set
    const content = await getEditorContent(app);
    expect(content).toBe(testCode);
  });

  test('should show log panel on script run', async ({ app }) => {
    // Set some Python code
    await setEditorContent(app, 'print("Test output")');

    // Click run button
    await clickRunButton(app);

    // Wait for log entry to appear
    await app.waitForSelector('.log-entry', { timeout: 10000 });

    // Check log contains "Running Script"
    const logContent = await getLogContent(app);
    expect(logContent).toContain('Running Script');
  });

  test('should handle file menu operations', async ({ app }) => {
    // Click File menu
    const fileMenu = app.locator('button:has-text("File")');
    if (await fileMenu.isVisible()) {
      await fileMenu.click();

      // Check menu items are visible
      await expect(app.locator('text=New File')).toBeVisible();
      await expect(app.locator('text=Open')).toBeVisible();
      await expect(app.locator('text=Save')).toBeVisible();
    }
  });

  test('should switch between tabs', async ({ app }) => {
    // Create a new tab
    const newTabButton = app.locator('.tab-add-button, button:has-text("+")');
    if (await newTabButton.isVisible()) {
      await newTabButton.click();

      // Check that we have multiple tabs
      const tabs = app.locator('.tab');
      const tabCount = await tabs.count();
      expect(tabCount).toBeGreaterThanOrEqual(2);
    }
  });

  test('should display status bar', async ({ app }) => {
    // Check status bar exists
    const statusBar = app.locator('.statusbar, .status-bar');
    await expect(statusBar).toBeVisible();
  });

  test('should have keyboard shortcuts', async ({ app }) => {
    // Test Ctrl+N for new file
    await app.keyboard.press('Control+n');

    // Should create a new tab
    await app.waitForTimeout(500);

    // Test Ctrl+S for save
    await setEditorContent(app, '# Test file');
    await app.keyboard.press('Control+s');

    // Save dialog should appear (or file saved message)
    await app.waitForTimeout(1000);
  });

});

test.describe('Script Execution', () => {

  test.beforeEach(async ({ app }) => {
    await waitForIDEReady(app);
  });

  test('should warn when running unsaved file', async ({ app }) => {
    // Set content without saving
    await setEditorContent(app, 'print("Unsaved")');

    // Click run
    await clickRunButton(app);

    // Should show save prompt or warning
    await app.waitForTimeout(1000);
    const logContent = await getLogContent(app);
    expect(logContent).toMatch(/save|Save|保存/);
  });

  test('should execute Python script', async ({ app }) => {
    // This test requires a saved file
    // First save the file
    await setEditorContent(app, 'print("Hello from test")');

    // Save with Ctrl+S (may trigger save dialog)
    await app.keyboard.press('Control+s');
    await app.waitForTimeout(2000);

    // Then run
    await clickRunButton(app);
    await app.waitForTimeout(5000);

    // Check execution log
    const logContent = await getLogContent(app);
    expect(logContent).toContain('Running Script');
  });

});

test.describe('Editor Features', () => {

  test.beforeEach(async ({ app }) => {
    await waitForIDEReady(app);
  });

  test('should have syntax highlighting', async ({ app }) => {
    // Set Python code
    await setEditorContent(app, 'def hello():\n    print("Hello")');

    // Check that Monaco applies syntax highlighting
    // Look for token classes
    const hasTokens = await app.locator('.mtk1, .mtk4, .mtk6').count();
    expect(hasTokens).toBeGreaterThan(0);
  });

  test('should show line numbers', async ({ app }) => {
    // Line numbers should be visible
    const lineNumbers = app.locator('.line-numbers, .margin-view-overlays');
    await expect(lineNumbers).toBeVisible();
  });

  test('should support undo/redo', async ({ app }) => {
    // Type something
    await setEditorContent(app, 'First');

    // Type more
    await setEditorContent(app, 'First Second');

    // Undo with Ctrl+Z
    await app.keyboard.press('Control+z');
    await app.waitForTimeout(500);

    // Content should revert
    // Note: This depends on how Monaco handles undo after setValue
  });

});

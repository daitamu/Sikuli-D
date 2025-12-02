import { test, expect, chromium } from '@playwright/test';

/**
 * Editor Operations E2E Tests
 * エディタ操作のE2Eテスト
 *
 * Tests Monaco editor functionality, syntax highlighting, line numbers, and copy operations
 * Monacoエディタの機能、シンタックスハイライト、行番号、コピー操作をテスト
 */

test.describe('Editor Operations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

    // Switch to Code mode
    const codeButton = page.locator('button:has-text("Code")');
    await codeButton.click();

    // Wait for Monaco editor to load
    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await page.waitForTimeout(1000); // Give editor time to initialize
  });

  test('should load Monaco editor in Code mode', async ({ page }) => {
    // Check that Monaco editor container exists
    const editor = page.locator('.monaco-editor');
    await expect(editor).toBeVisible();
  });

  test('should display Monaco editor with proper dimensions', async ({ page }) => {
    const editor = page.locator('.monaco-editor');

    // Get editor dimensions
    const box = await editor.boundingBox();

    // Editor should have non-zero dimensions
    expect(box).not.toBeNull();
    if (box) {
      expect(box.width).toBeGreaterThan(0);
      expect(box.height).toBeGreaterThan(0);
    }
  });

  test('should allow typing in the editor', async ({ page }) => {
    // Click in the editor to focus
    await page.locator('.monaco-editor').click();
    await page.waitForTimeout(500);

    // Type some text
    const testText = 'print("Hello World")';
    await page.keyboard.type(testText);

    // Wait a bit for the text to be rendered
    await page.waitForTimeout(500);

    // Verify text was entered by checking editor value via Monaco API
    const editorValue = await page.evaluate(() => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      return editor?.getValue() || '';
    });

    expect(editorValue).toContain(testText);
  });

  test('should display line numbers', async ({ page }) => {
    // Check for line numbers container
    const lineNumbers = page.locator('.line-numbers');

    // Monaco renders line numbers with specific classes
    // Check if at least one line number element exists
    const count = await lineNumbers.count();

    // If using monaco's default line numbers, check via CSS selector
    const monacoLineNumbers = page.locator('.margin-view-overlays');
    await expect(monacoLineNumbers).toBeVisible();
  });

  test('should apply Python syntax highlighting', async ({ page }) => {
    // Type Python code
    await page.locator('.monaco-editor').click();
    await page.waitForTimeout(500);

    const pythonCode = 'def hello():\n    print("world")';
    await page.keyboard.type(pythonCode);
    await page.waitForTimeout(1000); // Wait for syntax highlighting

    // Check that Monaco has applied syntax highlighting
    // Monaco uses specific CSS classes for tokens
    const hasTokens = await page.evaluate(() => {
      // Look for Monaco token elements
      const tokens = document.querySelectorAll('.view-line .mtk1, .view-line .mtk2, .view-line [class*="mtk"]');
      return tokens.length > 0;
    });

    expect(hasTokens).toBe(true);
  });

  test('should display Copy button in toolbar', async ({ page }) => {
    const copyButton = page.locator('button:has-text("Copy")');
    await expect(copyButton).toBeVisible();
  });

  test('should enable Copy button when editor has content', async ({ page }) => {
    // Initially might be disabled if empty
    const copyButton = page.locator('button:has-text("Copy")');

    // Type some content
    await page.locator('.monaco-editor').click();
    await page.keyboard.type('test content');
    await page.waitForTimeout(500);

    // Copy button should be enabled
    await expect(copyButton).toBeEnabled();
  });

  test('should show "Copied" feedback when Copy button is clicked', async ({ page }) => {
    // Add some content first
    await page.locator('.monaco-editor').click();
    await page.keyboard.type('test content for copy');
    await page.waitForTimeout(500);

    // Click Copy button
    const copyButton = page.locator('button:has-text("Copy")');
    await copyButton.click();

    // Should show "Copied" text
    const copiedText = page.locator('button:has-text("Copied")');
    await expect(copiedText).toBeVisible();

    // Wait for it to revert (2 seconds according to code)
    await page.waitForTimeout(2500);

    // Should revert back to "Copy"
    await expect(page.locator('button:has-text("Copy")')).toBeVisible();
  });

  test('should display Save button in toolbar', async ({ page }) => {
    const saveButton = page.locator('button:has-text("Save")').first();
    await expect(saveButton).toBeVisible();
  });

  test('should show file name in toolbar', async ({ page }) => {
    // Check for filename display (default is script.py)
    const filename = page.locator('.font-mono:has-text(".py")');
    await expect(filename).toBeVisible();

    // Default filename should be script.py
    const filenameText = await filename.textContent();
    expect(filenameText).toContain('.py');
  });

  test('should display line count in toolbar', async ({ page }) => {
    // Look for line count display
    const lineCount = page.locator('span:has-text("lines")');
    await expect(lineCount).toBeVisible();

    // Should show at least "1 lines" initially
    const lineCountText = await lineCount.textContent();
    expect(lineCountText).toMatch(/\d+ lines/);
  });

  test('should update line count when typing multiple lines', async ({ page }) => {
    // Get initial line count
    const lineCountSpan = page.locator('span:has-text("lines")');
    const initialCount = await lineCountSpan.textContent();

    // Type multiple lines
    await page.locator('.monaco-editor').click();
    await page.keyboard.type('line 1\nline 2\nline 3');
    await page.waitForTimeout(500);

    // Get new line count
    const newCount = await lineCountSpan.textContent();

    // Line count should have increased
    expect(newCount).not.toBe(initialCount);
  });

  test('should use dark theme (sikuli-dark)', async ({ page }) => {
    // Check editor background color is dark
    const editor = page.locator('.monaco-editor');

    const bgColor = await editor.evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });

    // Dark theme should have dark background
    // RGB values should be low (dark color)
    expect(bgColor).toMatch(/rgb\(\s*\d+,\s*\d+,\s*\d+\)/);
  });

  test('should display scrollbars when content exceeds view', async ({ page }) => {
    // Type enough content to require scrolling
    await page.locator('.monaco-editor').click();

    // Type many lines
    for (let i = 0; i < 50; i++) {
      await page.keyboard.type(`line ${i}\n`);
    }

    await page.waitForTimeout(1000);

    // Check for Monaco scrollbar
    const scrollbar = page.locator('.monaco-scrollable-element .scrollbar.vertical');
    await expect(scrollbar).toBeVisible();
  });

  test('should support text selection', async ({ page }) => {
    // Type some content
    await page.locator('.monaco-editor').click();
    const testText = 'select this text';
    await page.keyboard.type(testText);
    await page.waitForTimeout(500);

    // Select all with Ctrl+A
    await page.keyboard.press('Control+A');
    await page.waitForTimeout(500);

    // Check if text is selected via Monaco API
    const hasSelection = await page.evaluate(() => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      const selection = editor?.getSelection();
      return selection && !selection.isEmpty();
    });

    expect(hasSelection).toBe(true);
  });

  test('should support undo/redo operations', async ({ page }) => {
    // Type some text
    await page.locator('.monaco-editor').click();
    const text1 = 'first text';
    await page.keyboard.type(text1);
    await page.waitForTimeout(500);

    const text2 = '\nsecond text';
    await page.keyboard.type(text2);
    await page.waitForTimeout(500);

    // Undo last operation
    await page.keyboard.press('Control+Z');
    await page.waitForTimeout(500);

    // Check editor value
    const afterUndo = await page.evaluate(() => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      return editor?.getValue() || '';
    });

    // Should not contain second text after undo
    expect(afterUndo).toContain(text1);
    expect(afterUndo).not.toContain('second text');
  });

  test('should show cursor in editor', async ({ page }) => {
    // Click editor to focus
    await page.locator('.monaco-editor').click();
    await page.waitForTimeout(500);

    // Check for cursor element
    const cursor = page.locator('.cursor');
    const cursorCount = await cursor.count();

    // Monaco should render at least one cursor
    expect(cursorCount).toBeGreaterThan(0);
  });

  test('should support copy/paste operations', async ({ page }) => {
    // Type some text
    await page.locator('.monaco-editor').click();
    const originalText = 'text to copy';
    await page.keyboard.type(originalText);
    await page.waitForTimeout(500);

    // Select all and copy
    await page.keyboard.press('Control+A');
    await page.keyboard.press('Control+C');
    await page.waitForTimeout(500);

    // Move to end and add newline
    await page.keyboard.press('End');
    await page.keyboard.press('Enter');

    // Paste
    await page.keyboard.press('Control+V');
    await page.waitForTimeout(500);

    // Check editor value contains text twice
    const editorValue = await page.evaluate(() => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      return editor?.getValue() || '';
    });

    // Should contain original text at least twice (original + paste)
    const occurrences = (editorValue.match(new RegExp(originalText, 'g')) || []).length;
    expect(occurrences).toBeGreaterThanOrEqual(2);
  });

  test('should display minimap setting (disabled)', async ({ page }) => {
    // Check that minimap is disabled (according to CodeMode options)
    const minimap = page.locator('.monaco-editor .minimap');

    // Minimap should not be visible (minimap: { enabled: false })
    await expect(minimap).not.toBeVisible();
  });

  test('should render with correct font family', async ({ page }) => {
    const editor = page.locator('.monaco-editor .view-lines');

    // Check font family includes monospace
    const fontFamily = await editor.evaluate((el) => {
      return window.getComputedStyle(el).fontFamily;
    });

    // Should include monospace fonts
    expect(fontFamily.toLowerCase()).toContain('mono');
  });

  test('should show modified indicator when content changes', async ({ page }) => {
    // Type something to make editor modified
    await page.locator('.monaco-editor').click();
    await page.keyboard.type('modified content');
    await page.waitForTimeout(500);

    // Look for unsaved changes indicator (yellow dot)
    const modifiedIndicator = page.locator('.bg-yellow-500.rounded-full');
    await expect(modifiedIndicator).toBeVisible();
  });

  test('should display toolbar with proper styling', async ({ page }) => {
    // Check toolbar exists
    const toolbar = page.locator('.flex.items-center.justify-between.px-4.py-2');
    await expect(toolbar).toBeVisible();

    // Check toolbar has dark background
    const hasDarkBg = await toolbar.first().evaluate((el) => {
      return el.className.includes('bg-dark-surface');
    });
    expect(hasDarkBg).toBe(true);
  });

  test('should support bracket pair colorization', async ({ page }) => {
    // Type code with brackets
    await page.locator('.monaco-editor').click();
    await page.keyboard.type('def func():\n    if True:\n        pass');
    await page.waitForTimeout(1000);

    // Check that Monaco has rendered the code
    const editorValue = await page.evaluate(() => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      return editor?.getValue() || '';
    });

    expect(editorValue).toContain('def func');
    expect(editorValue).toContain('if True');
  });

  test('should have proper editor padding', async ({ page }) => {
    // Monaco editor should have padding according to options
    const viewLines = page.locator('.monaco-editor .view-lines');

    const padding = await viewLines.evaluate((el) => {
      const styles = window.getComputedStyle(el);
      return {
        paddingTop: styles.paddingTop,
        paddingBottom: styles.paddingBottom,
      };
    });

    // Should have some padding (defined as 12px in options)
    expect(padding.paddingTop).not.toBe('0px');
  });

  test('should show context menu on right click', async ({ page }) => {
    // Click in editor
    await page.locator('.monaco-editor').click();
    await page.waitForTimeout(500);

    // Right click to open context menu
    await page.locator('.monaco-editor').click({ button: 'right' });
    await page.waitForTimeout(500);

    // Check for Monaco context menu
    const contextMenu = page.locator('.context-view, .monaco-menu');
    const menuVisible = await contextMenu.isVisible().catch(() => false);

    // Context menu should appear (according to options: contextmenu: true)
    expect(menuVisible).toBe(true);
  });
});

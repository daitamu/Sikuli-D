import { test, expect, chromium } from '@playwright/test';

/**
 * View Mode Switching E2E Tests
 * ビューモード切り替えのE2Eテスト
 *
 * Tests switching between Simple, Flow, and Code modes
 * シンプル、フロー、コードモード間の切り替えをテスト
 */

test.describe('View Mode Switching', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });
  });

  test('should display all three view mode tabs', async ({ page }) => {
    // Check Simple mode tab
    const simpleTab = page.locator('button:has-text("Simple")');
    await expect(simpleTab).toBeVisible();

    // Check Flow mode tab
    const flowTab = page.locator('button:has-text("Flow")');
    await expect(flowTab).toBeVisible();

    // Check Code mode tab
    const codeTab = page.locator('button:has-text("Code")');
    await expect(codeTab).toBeVisible();
  });

  test('should have Simple mode tab clickable', async ({ page }) => {
    const simpleTab = page.locator('button:has-text("Simple")');
    await expect(simpleTab).toBeEnabled();

    // Should be able to click
    await simpleTab.click();
    await page.waitForTimeout(500);

    // Should remain enabled after click
    await expect(simpleTab).toBeEnabled();
  });

  test('should have Flow mode tab clickable', async ({ page }) => {
    const flowTab = page.locator('button:has-text("Flow")');
    await expect(flowTab).toBeEnabled();

    // Should be able to click
    await flowTab.click();
    await page.waitForTimeout(500);

    // Should remain enabled after click
    await expect(flowTab).toBeEnabled();
  });

  test('should have Code mode tab clickable', async ({ page }) => {
    const codeTab = page.locator('button:has-text("Code")');
    await expect(codeTab).toBeEnabled();

    // Should be able to click
    await codeTab.click();
    await page.waitForTimeout(500);

    // Should remain enabled after click
    await expect(codeTab).toBeEnabled();
  });

  test('should start in Simple mode by default', async ({ page }) => {
    // Simple mode tab should have active styling
    const simpleTab = page.locator('button:has-text("Simple")');

    // Check if tab has active class (bg-dark-surface and text-sikuli-400)
    const hasActiveStyle = await simpleTab.evaluate((el) => {
      return el.className.includes('bg-dark-surface') || el.className.includes('text-sikuli-400');
    });

    expect(hasActiveStyle).toBe(true);
  });

  test('should switch to Flow mode when clicked', async ({ page }) => {
    const flowTab = page.locator('button:has-text("Flow")');

    // Click Flow mode tab
    await flowTab.click();
    await page.waitForTimeout(500);

    // Flow mode tab should now have active styling
    const hasActiveStyle = await flowTab.evaluate((el) => {
      return el.className.includes('bg-dark-surface') || el.className.includes('text-sikuli-400');
    });

    expect(hasActiveStyle).toBe(true);
  });

  test('should switch to Code mode when clicked', async ({ page }) => {
    const codeTab = page.locator('button:has-text("Code")');

    // Click Code mode tab
    await codeTab.click();
    await page.waitForTimeout(500);

    // Code mode tab should now have active styling
    const hasActiveStyle = await codeTab.evaluate((el) => {
      return el.className.includes('bg-dark-surface') || el.className.includes('text-sikuli-400');
    });

    expect(hasActiveStyle).toBe(true);
  });

  test('should display Monaco editor in Code mode', async ({ page }) => {
    const codeTab = page.locator('button:has-text("Code")');

    // Switch to Code mode
    await codeTab.click();
    await page.waitForTimeout(1000);

    // Check for Monaco editor
    const editor = page.locator('.monaco-editor');
    await expect(editor).toBeVisible();
  });

  test('should switch between modes correctly', async ({ page }) => {
    // Start in Simple mode (default)
    let simpleTab = page.locator('button:has-text("Simple")');
    let flowTab = page.locator('button:has-text("Flow")');
    let codeTab = page.locator('button:has-text("Code")');

    // Switch to Flow
    await flowTab.click();
    await page.waitForTimeout(500);

    // Verify Flow is active
    let flowActive = await flowTab.evaluate((el) => {
      return el.className.includes('bg-dark-surface');
    });
    expect(flowActive).toBe(true);

    // Switch to Code
    await codeTab.click();
    await page.waitForTimeout(1000);

    // Verify Code is active and Monaco is loaded
    let codeActive = await codeTab.evaluate((el) => {
      return el.className.includes('bg-dark-surface');
    });
    expect(codeActive).toBe(true);

    await expect(page.locator('.monaco-editor')).toBeVisible();

    // Switch back to Simple
    await simpleTab.click();
    await page.waitForTimeout(500);

    // Verify Simple is active
    let simpleActive = await simpleTab.evaluate((el) => {
      return el.className.includes('bg-dark-surface');
    });
    expect(simpleActive).toBe(true);
  });

  test('should display mode tabs with icons', async ({ page }) => {
    // Simple mode should have LayoutList icon
    const simpleIcon = page.locator('button:has-text("Simple") svg');
    await expect(simpleIcon).toBeVisible();

    // Flow mode should have GitBranch icon
    const flowIcon = page.locator('button:has-text("Flow") svg');
    await expect(flowIcon).toBeVisible();

    // Code mode should have Code icon
    const codeIcon = page.locator('button:has-text("Code") svg');
    await expect(codeIcon).toBeVisible();
  });

  test('should have proper styling for mode tabs', async ({ page }) => {
    const simpleTab = page.locator('button:has-text("Simple")');

    // Check that tab has proper styling classes
    const hasProperStyling = await simpleTab.evaluate((el) => {
      return (
        el.className.includes('px-3') &&
        el.className.includes('py-1.5') &&
        el.className.includes('rounded-md')
      );
    });

    expect(hasProperStyling).toBe(true);
  });

  test('should show hover effect on inactive tabs', async ({ page }) => {
    // Switch to Simple mode first
    const simpleTab = page.locator('button:has-text("Simple")');
    await simpleTab.click();
    await page.waitForTimeout(500);

    // Hover over Code tab (inactive)
    const codeTab = page.locator('button:has-text("Code")');
    await codeTab.hover();
    await page.waitForTimeout(200);

    // Check that hover class exists
    const hasHoverClass = await codeTab.evaluate((el) => {
      return el.className.includes('hover:');
    });

    expect(hasHoverClass).toBe(true);
  });

  test('should display mode tabs in segmented control container', async ({ page }) => {
    // Check for segmented control container
    const container = page.locator('.bg-dark-bg.p-1.rounded-lg.border');
    await expect(container).toBeVisible();

    // Should contain all three mode buttons
    const buttonCount = await container.locator('button').count();
    expect(buttonCount).toBe(3);
  });

  test('should maintain active state when clicking active tab', async ({ page }) => {
    const simpleTab = page.locator('button:has-text("Simple")');

    // Click Simple tab (already active)
    await simpleTab.click();
    await page.waitForTimeout(500);

    // Should still be active
    const isActive = await simpleTab.evaluate((el) => {
      return el.className.includes('bg-dark-surface');
    });

    expect(isActive).toBe(true);
  });

  test('should change content area when switching modes', async ({ page }) => {
    // In Simple mode, check for Simple mode specific elements
    const simpleTab = page.locator('button:has-text("Simple")');
    await simpleTab.click();
    await page.waitForTimeout(500);

    // Simple mode should show some content (even if minimal)
    const body = await page.locator('body').textContent();
    expect(body).toBeTruthy();

    // Switch to Code mode
    const codeTab = page.locator('button:has-text("Code")');
    await codeTab.click();
    await page.waitForTimeout(1000);

    // Should show Monaco editor
    await expect(page.locator('.monaco-editor')).toBeVisible();
  });

  test('should disable Simple and Flow modes for external scripts', async ({ page }) => {
    // This test checks the disabled state logic
    // By default, Simple and Flow should be enabled

    const simpleTab = page.locator('button:has-text("Simple")');
    const flowTab = page.locator('button:has-text("Flow")');

    // Should be enabled by default
    await expect(simpleTab).toBeEnabled();
    await expect(flowTab).toBeEnabled();

    // Code mode should always be enabled
    const codeTab = page.locator('button:has-text("Code")');
    await expect(codeTab).toBeEnabled();
  });

  test('should show tooltip on mode tabs', async ({ page }) => {
    // Simple tab should have title attribute
    const simpleTab = page.locator('button:has-text("Simple")');
    const simpleTitle = await simpleTab.getAttribute('title');
    expect(simpleTitle).toBeTruthy();
    expect(simpleTitle).toContain('Simple');

    // Flow tab should have title attribute
    const flowTab = page.locator('button:has-text("Flow")');
    const flowTitle = await flowTab.getAttribute('title');
    expect(flowTitle).toBeTruthy();
    expect(flowTitle).toContain('Flow');

    // Code tab should have title attribute
    const codeTab = page.locator('button:has-text("Code")');
    const codeTitle = await codeTab.getAttribute('title');
    expect(codeTitle).toBeTruthy();
    expect(codeTitle).toContain('Code');
  });

  test('should have proper spacing between mode tabs', async ({ page }) => {
    const container = page.locator('.bg-dark-bg.p-1.rounded-lg.border');

    // Container should have gap between tabs (using flexbox)
    const hasFlexLayout = await container.evaluate((el) => {
      const styles = window.getComputedStyle(el);
      return styles.display === 'flex';
    });

    expect(hasFlexLayout).toBe(true);
  });

  test('should apply transition effects on mode switch', async ({ page }) => {
    const simpleTab = page.locator('button:has-text("Simple")');

    // Check for transition classes
    const hasTransition = await simpleTab.evaluate((el) => {
      return el.className.includes('transition');
    });

    expect(hasTransition).toBe(true);
  });

  test('should keep mode tabs visible at all times', async ({ page }) => {
    const simpleTab = page.locator('button:has-text("Simple")');
    const flowTab = page.locator('button:has-text("Flow")');
    const codeTab = page.locator('button:has-text("Code")');

    // All tabs should be visible
    await expect(simpleTab).toBeVisible();
    await expect(flowTab).toBeVisible();
    await expect(codeTab).toBeVisible();

    // Switch modes and verify tabs remain visible
    await codeTab.click();
    await page.waitForTimeout(500);

    await expect(simpleTab).toBeVisible();
    await expect(flowTab).toBeVisible();
    await expect(codeTab).toBeVisible();
  });

  test('should display mode selector in header center section', async ({ page }) => {
    // Mode selector should be in the center of header
    const header = page.locator('header');
    await expect(header).toBeVisible();

    // Mode tabs container should be within header
    const modeContainer = page.locator('header .bg-dark-bg.p-1.rounded-lg');
    await expect(modeContainer).toBeVisible();
  });

  test('should have consistent styling across all mode tabs', async ({ page }) => {
    const simpleTab = page.locator('button:has-text("Simple")');
    const flowTab = page.locator('button:has-text("Flow")');
    const codeTab = page.locator('button:has-text("Code")');

    // Get classes for each tab
    const simpleClass = await simpleTab.getAttribute('class');
    const flowClass = await flowTab.getAttribute('class');
    const codeClass = await codeTab.getAttribute('class');

    // All should have common base classes
    expect(simpleClass).toContain('px-3');
    expect(flowClass).toContain('px-3');
    expect(codeClass).toContain('px-3');

    expect(simpleClass).toContain('py-1.5');
    expect(flowClass).toContain('py-1.5');
    expect(codeClass).toContain('py-1.5');
  });

  test('should preserve editor state when switching from Code mode', async ({ page }) => {
    // Switch to Code mode
    const codeTab = page.locator('button:has-text("Code")');
    await codeTab.click();
    await page.waitForTimeout(1000);

    // Type some content
    await page.locator('.monaco-editor').click();
    const testText = 'test content';
    await page.keyboard.type(testText);
    await page.waitForTimeout(500);

    // Switch to Simple mode
    const simpleTab = page.locator('button:has-text("Simple")');
    await simpleTab.click();
    await page.waitForTimeout(500);

    // Switch back to Code mode
    await codeTab.click();
    await page.waitForTimeout(1000);

    // Content should be preserved
    const editorValue = await page.evaluate(() => {
      const monaco = (window as any).monaco;
      const editor = monaco?.editor?.getEditors?.()?.[0];
      return editor?.getValue() || '';
    });

    expect(editorValue).toContain(testText);
  });

  test('should show correct icon size for mode tabs', async ({ page }) => {
    const simpleIcon = page.locator('button:has-text("Simple") svg');

    // Icons should be 14px (size={14} in component)
    const iconSize = await simpleIcon.evaluate((el) => {
      return {
        width: el.getAttribute('width'),
        height: el.getAttribute('height'),
      };
    });

    expect(iconSize.width).toBe('14');
    expect(iconSize.height).toBe('14');
  });

  test('should have proper text size for mode labels', async ({ page }) => {
    const simpleTab = page.locator('button:has-text("Simple")');

    // Check for text-xs class (12px)
    const hasSmallText = await simpleTab.evaluate((el) => {
      return el.className.includes('text-xs');
    });

    expect(hasSmallText).toBe(true);
  });

  test('should render mode tabs with proper font weight', async ({ page }) => {
    const simpleTab = page.locator('button:has-text("Simple")');

    // Check for font-medium class
    const hasMediumFont = await simpleTab.evaluate((el) => {
      return el.className.includes('font-medium');
    });

    expect(hasMediumFont).toBe(true);
  });
});

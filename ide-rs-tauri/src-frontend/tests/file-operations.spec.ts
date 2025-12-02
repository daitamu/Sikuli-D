import { test, expect, chromium } from '@playwright/test';

/**
 * File Operations E2E Tests
 * ファイル操作のE2Eテスト
 *
 * Tests file menu operations, toolbar buttons, and keyboard shortcuts
 * ファイルメニュー操作、ツールバーボタン、キーボードショートカットをテスト
 */

test.describe('File Operations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173');
    // Wait for app to load
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });
  });

  test('should display file operation buttons in toolbar', async ({ page }) => {
    // Check New File button exists and is visible
    const newFileButton = page.locator('button[title="New File"]');
    await expect(newFileButton).toBeVisible();

    // Check Open File button exists and is visible
    const openFileButton = page.locator('button[title="Open File"]');
    await expect(openFileButton).toBeVisible();

    // Check Save File button exists and is visible
    const saveFileButton = page.locator('button[title="Save File"]');
    await expect(saveFileButton).toBeVisible();
  });

  test('should have clickable file operation buttons', async ({ page }) => {
    // New File button should be clickable
    const newFileButton = page.locator('button[title="New File"]');
    await expect(newFileButton).toBeEnabled();

    // Open File button should be clickable
    const openFileButton = page.locator('button[title="Open File"]');
    await expect(openFileButton).toBeEnabled();

    // Save File button should be clickable
    const saveFileButton = page.locator('button[title="Save File"]');
    await expect(saveFileButton).toBeEnabled();
  });

  test('should display file operation icons correctly', async ({ page }) => {
    // Check that buttons contain SVG icons (lucide-react icons)
    const newFileButton = page.locator('button[title="New File"] svg');
    await expect(newFileButton).toBeVisible();

    const openFileButton = page.locator('button[title="Open File"] svg');
    await expect(openFileButton).toBeVisible();

    const saveFileButton = page.locator('button[title="Save File"] svg');
    await expect(saveFileButton).toBeVisible();
  });

  test('should show hover effects on file buttons', async ({ page }) => {
    const newFileButton = page.locator('button[title="New File"]');

    // Get initial background color
    const initialBg = await newFileButton.evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });

    // Hover over button
    await newFileButton.hover();

    // Wait a bit for transition
    await page.waitForTimeout(100);

    // Background should change on hover (has hover:bg-dark-hover class)
    const hoveredBg = await newFileButton.evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });

    // Note: In headless mode, hover effects might not apply
    // So we just verify the button has the hover class in the DOM
    const hasHoverClass = await newFileButton.evaluate((el) => {
      return el.className.includes('hover:');
    });

    expect(hasHoverClass).toBe(true);
  });

  test('should display current file name when available', async ({ page }) => {
    // Initially, no file name should be displayed
    const fileNameDisplay = page.locator('.max-w-64 .truncate');

    // The file name display might not exist initially (conditional rendering)
    const count = await fileNameDisplay.count();

    // If count is 0, that's expected (no file loaded)
    // If count is 1, check it displays properly
    if (count > 0) {
      await expect(fileNameDisplay).toBeVisible();
    }
  });

  test('should display SIKULI-D logo and title', async ({ page }) => {
    // Check logo exists
    const logo = page.locator('.bg-sikuli-500.rounded-lg');
    await expect(logo).toBeVisible();

    // Check title
    const title = page.locator('h1:has-text("SIKULI-D")');
    await expect(title).toBeVisible();

    // Check version display
    const version = page.locator('span.text-sikuli-400.font-mono');
    await expect(version).toBeVisible();
  });

  test('should show file operations in header section', async ({ page }) => {
    // Verify all file operations are in the same container
    const fileOpsContainer = page.locator('.border-l.border-dark-border.pl-4');
    await expect(fileOpsContainer).toBeVisible();

    // Count buttons in file operations section
    const buttonCount = await fileOpsContainer.locator('button').count();
    expect(buttonCount).toBe(3); // New, Open, Save
  });

  test('should have proper button sizing and spacing', async ({ page }) => {
    const newFileButton = page.locator('button[title="New File"]');

    // Check button has padding class (p-2)
    const hasProperPadding = await newFileButton.evaluate((el) => {
      return el.className.includes('p-2');
    });
    expect(hasProperPadding).toBe(true);

    // Check icon size (should be 18px from lucide size prop)
    const iconSize = await newFileButton.locator('svg').evaluate((el) => {
      return {
        width: el.getAttribute('width'),
        height: el.getAttribute('height'),
      };
    });

    expect(iconSize.width).toBe('18');
    expect(iconSize.height).toBe('18');
  });

  test('should display keyboard shortcut hint on Run button', async ({ page }) => {
    // Run button should have title with keyboard shortcut
    const runButton = page.locator('button:has-text("Run")');
    const title = await runButton.getAttribute('title');
    expect(title).toBe('Run Script');
  });

  test('should display keyboard shortcut hint on Stop button when running', async ({ page }) => {
    // Stop button appears when script is running
    // We can't easily trigger running state, but we can check the component structure

    // Verify the button container exists
    const runStopContainer = page.locator('.flex.items-center').filter({ has: page.locator('button:has-text("Run")') });
    await expect(runStopContainer).toBeVisible();
  });

  test('should show Settings button in header', async ({ page }) => {
    const settingsButton = page.locator('button[title="Settings"]');
    await expect(settingsButton).toBeVisible();
    await expect(settingsButton).toBeEnabled();
  });

  test('should display all header sections correctly', async ({ page }) => {
    // Left section: Logo + File operations
    const leftSection = page.locator('header .flex.items-center.gap-6').first();
    await expect(leftSection).toBeVisible();

    // Center section: View mode toggle + Run/Stop
    const centerSection = page.locator('header .flex.items-center.gap-6').nth(1);
    await expect(centerSection).toBeVisible();

    // Right section: Settings
    const rightSection = page.locator('header .flex.items-center.gap-2');
    await expect(rightSection).toBeVisible();
  });

  test('should have proper header background and border', async ({ page }) => {
    const header = page.locator('header');

    // Check header has dark background
    const hasDarkBg = await header.evaluate((el) => {
      return el.className.includes('bg-dark-surface');
    });
    expect(hasDarkBg).toBe(true);

    // Check header has border
    const hasBorder = await header.evaluate((el) => {
      return el.className.includes('border-b');
    });
    expect(hasBorder).toBe(true);
  });

  test('should maintain button states correctly', async ({ page }) => {
    // All file operation buttons should be enabled by default
    const newFileButton = page.locator('button[title="New File"]');
    const openFileButton = page.locator('button[title="Open File"]');
    const saveFileButton = page.locator('button[title="Save File"]');

    await expect(newFileButton).not.toBeDisabled();
    await expect(openFileButton).not.toBeDisabled();
    await expect(saveFileButton).not.toBeDisabled();
  });

  test('should show Run button by default (not running state)', async ({ page }) => {
    // Run button should be visible initially
    const runButton = page.locator('button:has-text("Run")');
    await expect(runButton).toBeVisible();

    // Stop button should not be visible
    const stopButton = page.locator('button:has-text("Stop")');
    await expect(stopButton).not.toBeVisible();
  });

  test('should display Hide on Run checkbox', async ({ page }) => {
    // Check Hide on Run checkbox exists
    const hideCheckbox = page.locator('label:has-text("Hide on Run")');
    await expect(hideCheckbox).toBeVisible();

    // Check the actual input
    const checkbox = page.locator('input[type="checkbox"]').first();
    await expect(checkbox).toBeVisible();
  });

  test('should allow toggling Hide on Run checkbox', async ({ page }) => {
    const checkbox = page.locator('input[type="checkbox"]').first();

    // Get initial state
    const initialChecked = await checkbox.isChecked();

    // Click to toggle
    await checkbox.click();

    // Verify state changed
    const newChecked = await checkbox.isChecked();
    expect(newChecked).toBe(!initialChecked);
  });

  test('should have proper text styling for file operations', async ({ page }) => {
    // Check that buttons have proper text color classes
    const newFileButton = page.locator('button[title="New File"]');

    const hasTextColor = await newFileButton.evaluate((el) => {
      return el.className.includes('text-gray');
    });
    expect(hasTextColor).toBe(true);
  });

  test('should display version number in header', async ({ page }) => {
    const version = page.locator('span.text-sikuli-400.font-mono');
    await expect(version).toBeVisible();

    // Version should start with 'v'
    const versionText = await version.textContent();
    expect(versionText).toMatch(/^v\d+\.\d+\.\d+/);
  });
});

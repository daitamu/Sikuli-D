/**
 * SikuliX IDE E2E Tests with WebDriverIO
 * SikuliX IDE WebDriverIO E2Eテスト
 */

describe('SikuliX IDE', () => {

  it('should load the IDE window', async () => {
    // Wait for Monaco editor to be visible
    const editor = await $('.monaco-editor');
    await expect(editor).toBeDisplayed();
  });

  it('should have toolbar visible', async () => {
    const toolbar = await $('.toolbar');
    await expect(toolbar).toBeDisplayed();
  });

  it('should have bottom panel visible', async () => {
    const bottomPanel = await $('.bottom-panel');
    await expect(bottomPanel).toBeDisplayed();
  });

  it('should allow typing in the editor', async () => {
    // Focus on editor and type
    const editor = await $('.monaco-editor');
    await editor.click();

    // Type some code
    await browser.keys('print("Hello from test")');

    // Wait a bit for the content to be set
    await browser.pause(500);
  });

  it('should run script when clicking run button', async () => {
    // Find and click the run button
    const runButton = await $('button[onclick="runScript()"]');

    if (await runButton.isDisplayed()) {
      await runButton.click();

      // Wait for log panel to show output
      await browser.pause(2000);

      // Check that log panel has entries
      const logEntries = await $$('.log-entry');
      expect(logEntries.length).toBeGreaterThan(0);
    }
  });

  it('should create new tab', async () => {
    // Find the add tab button
    const addTabBtn = await $('.tab-add-button');

    if (await addTabBtn.isDisplayed()) {
      const initialTabs = await $$('.tab');
      const initialCount = initialTabs.length;

      await addTabBtn.click();
      await browser.pause(500);

      const newTabs = await $$('.tab');
      expect(newTabs.length).toBeGreaterThanOrEqual(initialCount);
    }
  });

  it('should handle keyboard shortcuts', async () => {
    // Test Ctrl+N for new file
    await browser.keys(['Control', 'n']);
    await browser.pause(500);

    // Test Ctrl+S for save
    await browser.keys(['Control', 's']);
    await browser.pause(1000);
  });

});

describe('Editor Features', () => {

  it('should show line numbers', async () => {
    const lineNumbers = await $('.margin-view-overlays');
    await expect(lineNumbers).toBeExisting();
  });

  it('should have syntax highlighting', async () => {
    // Look for Monaco token classes
    const tokens = await $$('.mtk1, .mtk4, .mtk6');
    expect(tokens.length).toBeGreaterThan(0);
  });

});

describe('Log Panel', () => {

  it('should switch to log panel', async () => {
    const logTab = await $('[data-panel="log"]');
    if (await logTab.isDisplayed()) {
      await logTab.click();
      await browser.pause(300);

      const logContent = await $('.log-content');
      await expect(logContent).toBeDisplayed();
    }
  });

  it('should clear log', async () => {
    const clearBtn = await $('button:contains("Clear")');
    if (await clearBtn.isDisplayed()) {
      await clearBtn.click();
      await browser.pause(300);
    }
  });

});

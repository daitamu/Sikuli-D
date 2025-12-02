# Sikuli-D IDE Frontend E2E Tests

Comprehensive Playwright end-to-end tests for the Sikuli-D IDE frontend.

## Overview

These tests validate the IDE frontend functionality against a Vite dev server, testing the web interface without requiring Tauri-specific APIs.

## Test Files

### 1. `file-operations.spec.ts`
Tests file operation UI elements and interactions:
- File menu button visibility and state
- Toolbar button functionality (New, Open, Save)
- Keyboard shortcut indicators
- File name display
- Header layout and styling
- Run/Stop button states
- Hide on Run checkbox

### 2. `editor-operations.spec.ts`
Tests Monaco editor functionality:
- Editor loading and rendering
- Text input and editing
- Syntax highlighting (Python)
- Line numbers display
- Copy/paste operations
- Undo/redo functionality
- Modified state indicators
- Toolbar features (Copy, Save buttons)
- Scrolling and text selection

### 3. `view-modes.spec.ts`
Tests view mode switching:
- Simple, Flow, and Code mode tabs
- Tab visibility and clickability
- Active state styling
- Mode switching functionality
- Content area updates
- Monaco editor loading in Code mode
- State preservation between mode switches
- Icon and text styling

## Prerequisites

1. **Node.js and npm** - Ensure Node.js 18+ is installed
2. **Dependencies installed** - Run `npm install` in `ide-rs-tauri/src-frontend`
3. **Playwright browsers** - Run `npx playwright install chromium`

## Running Tests

### 1. Start the Vite Dev Server

In one terminal:
```bash
cd ide-rs-tauri/src-frontend
npm run dev
```

Wait for the server to start on `http://localhost:5173`

### 2. Run Tests

In another terminal:

**Run all tests:**
```bash
cd ide-rs-tauri/src-frontend
npm test
```

**Run specific test file:**
```bash
npx playwright test file-operations.spec.ts
npx playwright test editor-operations.spec.ts
npx playwright test view-modes.spec.ts
```

**Run in headed mode (see browser):**
```bash
npm run test:headed
```

**Run with UI mode (interactive):**
```bash
npm run test:ui
```

**Debug mode:**
```bash
npm run test:debug
```

**Show test report:**
```bash
npm run test:report
```

## Test Configuration

Configuration is in `playwright.config.ts`:
- **Base URL**: `http://localhost:5173` (Vite dev server)
- **Timeout**: 30 seconds per test
- **Browser**: Chromium
- **Screenshots**: Captured on failure
- **Trace**: Recorded on retry
- **Web Server**: Automatically starts Vite if needed

## Test Structure

Each test file follows this pattern:

```typescript
test.describe('Feature Group', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to app and wait for it to load
    await page.goto('http://localhost:5173');
    await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });
  });

  test('should do something', async ({ page }) => {
    // Test assertions
    const element = page.locator('selector');
    await expect(element).toBeVisible();
  });
});
```

## Common Selectors

- **File buttons**: `button[title="New File"]`, `button[title="Open File"]`, `button[title="Save File"]`
- **Mode tabs**: `button:has-text("Simple")`, `button:has-text("Flow")`, `button:has-text("Code")`
- **Monaco editor**: `.monaco-editor`
- **Run/Stop buttons**: `button:has-text("Run")`, `button:has-text("Stop")`
- **Header**: `header`
- **Toolbar**: `.flex.items-center.justify-between.px-4.py-2`

## Debugging Failed Tests

1. **Check screenshots**: Failed tests save screenshots to `test-results/`
2. **View trace**: Use `npm run test:report` to see traces
3. **Run in headed mode**: `npm run test:headed` to see browser
4. **Use debug mode**: `npm run test:debug` for step-by-step debugging

## Known Limitations

1. **No Tauri APIs**: These tests run against the Vite dev server, so Tauri-specific features (file dialogs, IPC, etc.) are mocked or not available
2. **Hover effects**: CSS hover effects may not always trigger in headless mode
3. **Timing**: Some tests use `waitForTimeout` for animations/transitions
4. **Monaco initialization**: Editor needs time to load, tests wait 1 second after switching to Code mode

## Testing Best Practices

1. **Wait for elements**: Always use `waitForSelector` or `expect().toBeVisible()` instead of fixed delays
2. **Unique selectors**: Use specific selectors like `[title="..."]` or `:has-text("...")`
3. **Test isolation**: Each test should be independent and not rely on previous test state
4. **Descriptive names**: Test names should clearly describe what is being tested
5. **Multiple assertions**: Group related assertions in a single test for efficiency

## CI/CD Integration

Add to your CI pipeline:

```yaml
- name: Install dependencies
  run: cd ide-rs-tauri/src-frontend && npm install

- name: Install Playwright browsers
  run: cd ide-rs-tauri/src-frontend && npx playwright install chromium

- name: Run tests
  run: cd ide-rs-tauri/src-frontend && npm test
```

## Troubleshooting

**Test hangs waiting for selector:**
- Verify Vite dev server is running on port 5173
- Check browser console for errors: `npm run test:headed`

**Monaco editor not loading:**
- Increase timeout in `waitForSelector('.monaco-editor', { timeout: 10000 })`
- Ensure @monaco-editor/react is installed

**Typing doesn't work in editor:**
- Ensure editor is focused with `.click()` before typing
- Add `waitForTimeout(500)` after click to ensure focus

**Mode switching tests fail:**
- Verify the tab has the correct text: "Simple", "Flow", "Code" (case-sensitive)
- Check that tabs are not disabled

## Contributing

When adding new tests:
1. Follow existing test structure
2. Add clear comments in both English and Japanese
3. Use descriptive test names
4. Wait for elements properly (avoid fixed delays when possible)
5. Test both positive and negative cases
6. Update this README with new test descriptions

## Related Files

- `playwright.config.ts` - Playwright configuration
- `package.json` - Test scripts
- `src/` - Frontend source code being tested
- `test-results/` - Test output (screenshots, traces)

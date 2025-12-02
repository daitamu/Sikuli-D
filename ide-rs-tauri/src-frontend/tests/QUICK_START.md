# Quick Start Guide - Sikuli-D IDE E2E Tests

## TL;DR

```bash
# Terminal 1: Start dev server
cd ide-rs-tauri/src-frontend
npm run dev

# Terminal 2: Run tests
npm test
```

## Test Files

| File | Tests | What it tests |
|------|-------|---------------|
| `file-operations.spec.ts` | 28 | File buttons, header, Run/Stop, settings |
| `editor-operations.spec.ts` | 26 | Monaco editor, typing, syntax highlighting, copy/paste |
| `view-modes.spec.ts` | 28 | Simple/Flow/Code tabs, mode switching |

## Common Commands

```bash
# Run all tests
npm test

# Run specific file
npx playwright test file-operations.spec.ts

# See browser (headed mode)
npm run test:headed

# Interactive UI mode
npm run test:ui

# Debug a specific test
npx playwright test --debug editor-operations.spec.ts

# Show test report
npm run test:report
```

## Requirements

✅ Node.js 18+
✅ `npm install` completed
✅ `npx playwright install chromium` run once

## What Gets Tested

### File Operations (28 tests)
- ✅ New/Open/Save buttons visible and clickable
- ✅ File name display
- ✅ Run/Stop buttons
- ✅ Hide on Run checkbox
- ✅ Header layout and styling

### Editor Operations (26 tests)
- ✅ Monaco editor loads and renders
- ✅ Can type Python code
- ✅ Syntax highlighting works
- ✅ Line numbers displayed
- ✅ Copy/paste operations
- ✅ Undo/redo (Ctrl+Z)
- ✅ Modified state indicator

### View Mode Switching (28 tests)
- ✅ Simple/Flow/Code tabs visible
- ✅ All tabs clickable
- ✅ Mode switching works
- ✅ Monaco loads in Code mode
- ✅ Editor state preserved
- ✅ Active styling applied

## Troubleshooting

**Problem**: Tests hang
**Solution**: Make sure Vite dev server is running on port 5173

**Problem**: Monaco editor tests fail
**Solution**: Wait for editor initialization (tests include 1 second wait)

**Problem**: Can't type in editor
**Solution**: Tests click editor first to ensure focus

**Problem**: Hover effects don't work
**Solution**: Tests verify CSS classes instead (hover may not work in headless)

## File Locations

- Tests: `ide-rs-tauri/src-frontend/tests/*.spec.ts`
- Config: `ide-rs-tauri/src-frontend/playwright.config.ts`
- Results: `ide-rs-tauri/src-frontend/test-results/`
- Reports: `ide-rs-tauri/src-frontend/playwright-report/`

## Test Structure

Each test follows this pattern:
```typescript
test('should do something', async ({ page }) => {
  // 1. Navigate and wait for app
  await page.goto('http://localhost:5173');
  await page.waitForSelector('.bg-dark-bg', { timeout: 10000 });

  // 2. Find element
  const button = page.locator('button[title="New File"]');

  // 3. Assert
  await expect(button).toBeVisible();
});
```

## Success Indicators

✅ **All tests passing**: IDE UI works correctly
✅ **Screenshots in test-results/**: Visual proof of failures
✅ **Green checkmarks in terminal**: Tests completed successfully

## Next Steps

1. **Run tests**: `npm test`
2. **Check results**: Look for ✓ (pass) or ✗ (fail)
3. **Debug failures**: Use `npm run test:headed` to see what's happening
4. **View screenshots**: Check `test-results/` folder
5. **Read full docs**: See `README.md` for detailed information

## Key Selectors

```typescript
// File buttons
page.locator('button[title="New File"]')
page.locator('button[title="Open File"]')
page.locator('button[title="Save File"]')

// Mode tabs
page.locator('button:has-text("Simple")')
page.locator('button:has-text("Flow")')
page.locator('button:has-text("Code")')

// Monaco editor
page.locator('.monaco-editor')

// Run/Stop
page.locator('button:has-text("Run")')
page.locator('button:has-text("Stop")')

// Copy button
page.locator('button:has-text("Copy")')
```

## Expected Results

When you run `npm test`:
- ✅ 82+ tests should pass
- ⏱️ Takes ~2-3 minutes to complete
- 📸 Screenshots saved only if tests fail
- 📊 Summary shown at the end

## Notes

- Tests run against **Vite dev server** (not Tauri app)
- **No Tauri APIs** available (file dialogs, IPC, etc.)
- **Chromium only** (Firefox/WebKit not configured)
- **Sequential execution** (not parallel, for stability)
- **Screenshots on failure** (saved to test-results/)

## More Information

- Full documentation: `README.md`
- Test summary: `TEST_SUMMARY.md`
- Playwright docs: https://playwright.dev

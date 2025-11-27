/**
 * Playwright fixtures for Tauri app testing
 * Tauriアプリテスト用Playwrightフィクスチャ
 */

import { test as base, Page, BrowserContext, chromium } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';

// Path to the built Tauri app
const APP_PATH = path.resolve(__dirname, '../../target/debug/sikulix-ide-tauri.exe');
const TAURI_DRIVER_PATH = path.resolve(process.env.USERPROFILE || '', '.cargo/bin/tauri-driver.exe');

interface TauriFixtures {
  app: Page;
  tauriContext: BrowserContext;
}

let tauriDriverProcess: ChildProcess | null = null;
let appProcess: ChildProcess | null = null;

/**
 * Start tauri-driver WebDriver server
 * tauri-driver WebDriverサーバーを起動
 */
async function startTauriDriver(): Promise<void> {
  return new Promise((resolve, reject) => {
    tauriDriverProcess = spawn(TAURI_DRIVER_PATH, [], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    tauriDriverProcess.stdout?.on('data', (data) => {
      const output = data.toString();
      console.log('[tauri-driver]', output);
      if (output.includes('Listening on')) {
        resolve();
      }
    });

    tauriDriverProcess.stderr?.on('data', (data) => {
      console.error('[tauri-driver error]', data.toString());
    });

    tauriDriverProcess.on('error', reject);

    // Timeout after 10 seconds
    setTimeout(() => resolve(), 3000);
  });
}

/**
 * Stop tauri-driver
 * tauri-driverを停止
 */
function stopTauriDriver(): void {
  if (tauriDriverProcess) {
    tauriDriverProcess.kill();
    tauriDriverProcess = null;
  }
}

/**
 * Extended test with Tauri fixtures
 * Tauriフィクスチャ付き拡張テスト
 */
export const test = base.extend<TauriFixtures>({
  tauriContext: async ({}, use) => {
    // Start tauri-driver
    await startTauriDriver();

    // Connect to WebDriver
    const browser = await chromium.connectOverCDP('http://localhost:4444');
    const context = browser.contexts()[0] || await browser.newContext();

    await use(context);

    // Cleanup
    await browser.close();
    stopTauriDriver();
  },

  app: async ({ tauriContext }, use) => {
    const pages = tauriContext.pages();
    const page = pages[0] || await tauriContext.newPage();

    // Wait for app to be ready
    await page.waitForLoadState('domcontentloaded');

    await use(page);
  },
});

export { expect } from '@playwright/test';

/**
 * Helper to wait for the IDE to be fully loaded
 * IDEが完全に読み込まれるのを待つヘルパー
 */
export async function waitForIDEReady(page: Page): Promise<void> {
  // Wait for Monaco editor to be initialized
  await page.waitForSelector('.monaco-editor', { timeout: 30000 });

  // Wait for the toolbar to be visible
  await page.waitForSelector('.toolbar', { timeout: 10000 });
}

/**
 * Helper to get editor content
 * エディタの内容を取得するヘルパー
 */
export async function getEditorContent(page: Page): Promise<string> {
  return await page.evaluate(() => {
    // @ts-ignore - monacoEditor is a global variable
    return window.monacoEditor?.getValue() || '';
  });
}

/**
 * Helper to set editor content
 * エディタの内容を設定するヘルパー
 */
export async function setEditorContent(page: Page, content: string): Promise<void> {
  await page.evaluate((text) => {
    // @ts-ignore - monacoEditor is a global variable
    window.monacoEditor?.setValue(text);
  }, content);
}

/**
 * Helper to click run button
 * 実行ボタンをクリックするヘルパー
 */
export async function clickRunButton(page: Page): Promise<void> {
  await page.click('button[onclick="runScript()"]');
}

/**
 * Helper to get log panel content
 * ログパネルの内容を取得するヘルパー
 */
export async function getLogContent(page: Page): Promise<string> {
  const logEntries = await page.$$eval('.log-entry', entries =>
    entries.map(e => e.textContent || '').join('\n')
  );
  return logEntries;
}

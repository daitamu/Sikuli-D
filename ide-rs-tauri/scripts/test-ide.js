#!/usr/bin/env node
/**
 * SikuliX IDE Automated Test Script
 * SikuliX IDE 自動テストスクリプト
 *
 * This script connects to a running IDE via tauri-driver and performs automated tests.
 * このスクリプトはtauri-driver経由で実行中のIDEに接続し、自動テストを実行します。
 *
 * Prerequisites / 前提条件:
 *   1. Build the app: cargo build
 *   2. Start tauri-driver: tauri-driver (in separate terminal)
 *   3. Start the app: cargo tauri dev (or run the built exe)
 *   4. Run this script: node scripts/test-ide.js
 */

const http = require('http');

// WebDriver endpoint (tauri-driver default port)
const WD_HOST = 'localhost';
const WD_PORT = 4444;

/**
 * Make a WebDriver request
 */
async function wdRequest(method, path, body = null) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: WD_HOST,
      port: WD_PORT,
      path: path,
      method: method,
      headers: {
        'Content-Type': 'application/json',
      },
    };

    const req = http.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          resolve(data);
        }
      });
    });

    req.on('error', reject);

    if (body) {
      req.write(JSON.stringify(body));
    }

    req.end();
  });
}

/**
 * Test class for IDE automation
 */
class IDETest {
  constructor() {
    this.sessionId = null;
  }

  /**
   * Create a new WebDriver session
   */
  async createSession() {
    console.log('Creating WebDriver session...');

    const result = await wdRequest('POST', '/session', {
      capabilities: {
        alwaysMatch: {
          'tauri:options': {
            application: './target/debug/sikulix-ide-tauri.exe',
          },
        },
      },
    });

    if (result.value && result.value.sessionId) {
      this.sessionId = result.value.sessionId;
      console.log(`Session created: ${this.sessionId}`);
      return true;
    }

    console.error('Failed to create session:', result);
    return false;
  }

  /**
   * Delete the WebDriver session
   */
  async deleteSession() {
    if (this.sessionId) {
      await wdRequest('DELETE', `/session/${this.sessionId}`);
      console.log('Session deleted');
    }
  }

  /**
   * Find an element by CSS selector
   */
  async findElement(selector) {
    const result = await wdRequest('POST', `/session/${this.sessionId}/element`, {
      using: 'css selector',
      value: selector,
    });
    return result.value;
  }

  /**
   * Find multiple elements by CSS selector
   */
  async findElements(selector) {
    const result = await wdRequest('POST', `/session/${this.sessionId}/elements`, {
      using: 'css selector',
      value: selector,
    });
    return result.value || [];
  }

  /**
   * Click an element
   */
  async click(elementId) {
    await wdRequest('POST', `/session/${this.sessionId}/element/${elementId}/click`);
  }

  /**
   * Send keys to an element
   */
  async sendKeys(elementId, text) {
    await wdRequest('POST', `/session/${this.sessionId}/element/${elementId}/value`, {
      text: text,
    });
  }

  /**
   * Get element text
   */
  async getText(elementId) {
    const result = await wdRequest('GET', `/session/${this.sessionId}/element/${elementId}/text`);
    return result.value;
  }

  /**
   * Execute JavaScript in the browser context
   */
  async executeScript(script, args = []) {
    const result = await wdRequest('POST', `/session/${this.sessionId}/execute/sync`, {
      script: script,
      args: args,
    });
    return result.value;
  }

  /**
   * Take a screenshot
   */
  async takeScreenshot() {
    const result = await wdRequest('GET', `/session/${this.sessionId}/screenshot`);
    return result.value; // Base64 encoded PNG
  }

  /**
   * Wait for a specified time
   */
  async wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // ================== Test Methods ==================

  /**
   * Test: Check IDE loads correctly
   */
  async testIDELoads() {
    console.log('\n--- Test: IDE Loads ---');

    // Wait for app to initialize
    await this.wait(2000);

    // Check for Monaco editor
    const editors = await this.findElements('.monaco-editor');
    if (editors.length > 0) {
      console.log('✓ Monaco editor found');
      return true;
    } else {
      console.log('✗ Monaco editor not found');
      return false;
    }
  }

  /**
   * Test: Check toolbar exists
   */
  async testToolbarExists() {
    console.log('\n--- Test: Toolbar Exists ---');

    const toolbars = await this.findElements('.toolbar');
    if (toolbars.length > 0) {
      console.log('✓ Toolbar found');
      return true;
    } else {
      console.log('✗ Toolbar not found');
      return false;
    }
  }

  /**
   * Test: Type in editor
   */
  async testTypeInEditor() {
    console.log('\n--- Test: Type in Editor ---');

    // Set content via JavaScript
    const testCode = 'print("Hello from automated test!")';
    await this.executeScript(`
      if (window.monacoEditor) {
        window.monacoEditor.setValue(${JSON.stringify(testCode)});
        return true;
      }
      return false;
    `);

    await this.wait(500);

    // Verify content
    const content = await this.executeScript(`
      return window.monacoEditor ? window.monacoEditor.getValue() : '';
    `);

    if (content === testCode) {
      console.log('✓ Editor content set successfully');
      return true;
    } else {
      console.log('✗ Editor content mismatch');
      return false;
    }
  }

  /**
   * Test: Run script button
   */
  async testRunButton() {
    console.log('\n--- Test: Run Button ---');

    const buttons = await this.findElements('button[onclick="runScript()"]');
    if (buttons.length > 0) {
      console.log('✓ Run button found');
      // Click it
      await this.click(buttons[0]['element-6066-11e4-a52e-4f735466cecf'] || buttons[0].ELEMENT);
      await this.wait(1000);
      console.log('✓ Run button clicked');
      return true;
    } else {
      console.log('✗ Run button not found');
      return false;
    }
  }

  /**
   * Test: Check log panel
   */
  async testLogPanel() {
    console.log('\n--- Test: Log Panel ---');

    const logEntries = await this.findElements('.log-entry');
    console.log(`Found ${logEntries.length} log entries`);
    return true;
  }
}

/**
 * Main test runner
 */
async function main() {
  console.log('=====================================');
  console.log('SikuliX IDE Automated Test Runner');
  console.log('=====================================\n');

  const test = new IDETest();
  let passed = 0;
  let failed = 0;

  try {
    // Create session
    const sessionCreated = await test.createSession();
    if (!sessionCreated) {
      console.error('\nFailed to create WebDriver session.');
      console.error('Make sure:');
      console.error('  1. tauri-driver is running (run: tauri-driver)');
      console.error('  2. The app is built (run: cargo build)');
      process.exit(1);
    }

    // Run tests
    const tests = [
      () => test.testIDELoads(),
      () => test.testToolbarExists(),
      () => test.testTypeInEditor(),
      () => test.testRunButton(),
      () => test.testLogPanel(),
    ];

    for (const t of tests) {
      try {
        const result = await t();
        if (result) passed++;
        else failed++;
      } catch (e) {
        console.error('Test error:', e.message);
        failed++;
      }
    }

    // Take final screenshot
    console.log('\n--- Taking Screenshot ---');
    const screenshot = await test.takeScreenshot();
    if (screenshot) {
      const fs = require('fs');
      const path = require('path');
      const screenshotPath = path.join(__dirname, '..', 'test-screenshot.png');
      fs.writeFileSync(screenshotPath, Buffer.from(screenshot, 'base64'));
      console.log(`Screenshot saved to: ${screenshotPath}`);
    }

  } catch (error) {
    console.error('Test runner error:', error);
  } finally {
    await test.deleteSession();
  }

  // Summary
  console.log('\n=====================================');
  console.log('Test Summary / テスト結果');
  console.log('=====================================');
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  console.log(`Total:  ${passed + failed}`);

  process.exit(failed > 0 ? 1 : 0);
}

main();

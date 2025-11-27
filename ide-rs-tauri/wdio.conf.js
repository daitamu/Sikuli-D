/**
 * WebDriverIO configuration for SikuliX IDE E2E tests
 * SikuliX IDE E2Eテスト用WebDriverIO設定
 *
 * To run tests:
 *   1. Build the app: cargo build
 *   2. Start tauri-driver: tauri-driver
 *   3. Run tests: npm run wdio
 */

const path = require('path');

// Path to the built Tauri app
const appPath = path.resolve(__dirname, 'target/debug/sikulix-ide-tauri.exe');

exports.config = {
  // Runner Configuration
  runner: 'local',

  // Specs
  specs: [
    './tests/wdio/**/*.spec.js'
  ],
  exclude: [],

  // Capabilities - Tauri uses WebView2 which is Chromium-based
  capabilities: [{
    maxInstances: 1,
    'tauri:options': {
      application: appPath,
    },
    // Browser capabilities for WebView2
    browserName: 'chrome',
    'goog:chromeOptions': {
      // Attach to the Tauri WebView
      debuggerAddress: 'localhost:9222',
    },
  }],

  // Log level
  logLevel: 'info',

  // Base URL
  baseUrl: 'tauri://localhost',

  // Wait for timeout
  waitforTimeout: 10000,

  // Connection retry
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  // Services
  services: [],

  // Framework
  framework: 'mocha',
  reporters: ['spec'],

  // Mocha options
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000
  },

  // Hooks
  onPrepare: async function (config, capabilities) {
    console.log('Starting tauri-driver...');
    // tauri-driver should be started separately
  },

  before: async function (capabilities, specs) {
    console.log('Test suite starting...');
  },

  afterTest: async function (test, context, { error, result, duration, passed, retries }) {
    if (!passed) {
      console.log(`Test "${test.title}" failed`);
    }
  },
};

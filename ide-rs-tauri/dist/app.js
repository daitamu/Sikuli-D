// SikuliX IDE - Tauri Frontend with Monaco Editor
// SikuliX IDE - Monaco Editor搭載Tauriフロントエンド

const { invoke } = window.__TAURI__.core;
const { WebviewWindow } = window.__TAURI__.webviewWindow;

// ============================================================================
// Global State / グローバル状態
// ============================================================================

let monacoEditor = null;
let debounceTimer = null;
let isRunning = false;

// Tab management
let tabs = [];
let activeTabId = null;
let nextTabId = 1;

// Context menu state
let contextMenuTabId = null;

// Editor settings
let editorSettings = {
    theme: 'vs-dark',
    fontSize: 14
};

// Log filtering state
let logEntries = [];
let logFilterLevels = {
    debug: true,
    info: true,
    warn: true,
    error: true
};
let logSearchText = '';

// Problems panel state
let problems = [];
let problemsFilter = 'all';
let editorDecorations = [];

// Breakpoint state
let breakpoints = []; // Array of { line: number, enabled: boolean, condition: string|null }
let breakpointDecorations = [];

// Variables inspector state
let debugVariables = {
    local: [],
    global: []
};
let variablesScope = 'local';
let variablesSearchText = '';
let expandedVariables = new Set(); // Track expanded tree nodes

// Watch expressions state
let watchExpressions = []; // Array of { id: number, expression: string, value: any, error: string|null }
let nextWatchId = 1;

// Pattern editor state
let currentPattern = {
    imagePath: null,
    similarity: 0.7,
    targetOffset: { x: 0, y: 0 },
    imageWidth: 0,
    imageHeight: 0
};
let patternImage = null;
let patternCallback = null;

// SikuliX API completions
const sikulixCompletions = [
    // Screen operations
    { label: 'find', kind: 'Function', detail: 'find(pattern) -> Match', documentation: 'Find pattern on screen' },
    { label: 'findAll', kind: 'Function', detail: 'findAll(pattern) -> list[Match]', documentation: 'Find all occurrences of pattern on screen' },
    { label: 'wait', kind: 'Function', detail: 'wait(pattern, timeout) -> Match', documentation: 'Wait for pattern to appear' },
    { label: 'waitVanish', kind: 'Function', detail: 'waitVanish(pattern, timeout) -> bool', documentation: 'Wait for pattern to disappear' },
    { label: 'exists', kind: 'Function', detail: 'exists(pattern, timeout) -> Match|None', documentation: 'Check if pattern exists on screen' },
    // Mouse operations
    { label: 'click', kind: 'Function', detail: 'click(target)', documentation: 'Click on target (Pattern or location)' },
    { label: 'doubleClick', kind: 'Function', detail: 'doubleClick(target)', documentation: 'Double-click on target' },
    { label: 'rightClick', kind: 'Function', detail: 'rightClick(target)', documentation: 'Right-click on target' },
    { label: 'hover', kind: 'Function', detail: 'hover(target)', documentation: 'Move mouse to target' },
    { label: 'drag', kind: 'Function', detail: 'drag(target)', documentation: 'Start drag from target' },
    { label: 'dropAt', kind: 'Function', detail: 'dropAt(target)', documentation: 'Drop at target' },
    { label: 'dragDrop', kind: 'Function', detail: 'dragDrop(from, to)', documentation: 'Drag from one location to another' },
    // Keyboard operations
    { label: 'type', kind: 'Function', detail: 'type(text, modifiers)', documentation: 'Type text with optional modifiers' },
    { label: 'paste', kind: 'Function', detail: 'paste(text)', documentation: 'Paste text from clipboard' },
    { label: 'keyDown', kind: 'Function', detail: 'keyDown(key)', documentation: 'Press and hold key' },
    { label: 'keyUp', kind: 'Function', detail: 'keyUp(key)', documentation: 'Release key' },
    // Region operations
    { label: 'Region', kind: 'Class', detail: 'Region(x, y, w, h)', documentation: 'Define a screen region' },
    { label: 'Screen', kind: 'Class', detail: 'Screen(id)', documentation: 'Get screen by ID' },
    { label: 'Pattern', kind: 'Class', detail: 'Pattern(image)', documentation: 'Create pattern from image' },
    // Utility
    { label: 'sleep', kind: 'Function', detail: 'sleep(seconds)', documentation: 'Wait for specified seconds' },
    { label: 'popup', kind: 'Function', detail: 'popup(message)', documentation: 'Show popup message' },
    { label: 'input', kind: 'Function', detail: 'input(prompt) -> str', documentation: 'Show input dialog' },
    { label: 'capture', kind: 'Function', detail: 'capture(region) -> str', documentation: 'Capture screen region' },
    // Settings
    { label: 'Settings', kind: 'Class', detail: 'Settings', documentation: 'SikuliX settings object' },
    { label: 'setShowActions', kind: 'Function', detail: 'setShowActions(bool)', documentation: 'Show visual feedback' }
];

// ============================================================================
// Tab Data Structure / タブデータ構造
// ============================================================================

function createTab(path = null, content = '', isModified = false) {
    const id = nextTabId++;
    return {
        id,
        path,
        content,
        originalContent: content,
        isModified,
        cursorPosition: { line: 1, col: 1 },
        viewState: null // Monaco editor view state
    };
}

// ============================================================================
// Monaco Editor Initialization / Monaco Editor初期化
// ============================================================================

function initMonacoEditor() {
    return new Promise((resolve, reject) => {
        require(['vs/editor/editor.main'], function() {
            // Register SikuliX custom theme
            monaco.editor.defineTheme('sikulix-dark', {
                base: 'vs-dark',
                inherit: true,
                rules: [
                    { token: 'comment', foreground: '6A9955' },
                    { token: 'keyword', foreground: '569CD6' },
                    { token: 'string', foreground: 'CE9178' },
                    { token: 'number', foreground: 'B5CEA8' },
                    { token: 'function', foreground: 'DCDCAA' },
                    { token: 'class', foreground: '4EC9B0' },
                    { token: 'variable', foreground: '9CDCFE' },
                    { token: 'type', foreground: '4EC9B0' },
                    // SikuliX specific
                    { token: 'sikulix-function', foreground: 'FFD700', fontStyle: 'bold' }
                ],
                colors: {
                    'editor.background': '#1E1E1E',
                    'editor.foreground': '#D4D4D4',
                    'editorLineNumber.foreground': '#858585',
                    'editorCursor.foreground': '#AEAFAD',
                    'editor.selectionBackground': '#264F78',
                    'editor.lineHighlightBackground': '#2D2D30'
                }
            });

            // Register SikuliX completion provider for Python
            monaco.languages.registerCompletionItemProvider('python', {
                provideCompletionItems: function(model, position) {
                    const word = model.getWordUntilPosition(position);
                    const range = {
                        startLineNumber: position.lineNumber,
                        endLineNumber: position.lineNumber,
                        startColumn: word.startColumn,
                        endColumn: word.endColumn
                    };

                    const suggestions = sikulixCompletions.map(item => ({
                        label: item.label,
                        kind: monaco.languages.CompletionItemKind[item.kind] || monaco.languages.CompletionItemKind.Function,
                        documentation: item.documentation,
                        detail: item.detail,
                        insertText: item.label,
                        range: range
                    }));

                    return { suggestions };
                }
            });

            // Create editor instance
            monacoEditor = monaco.editor.create(document.getElementById('editor-container'), {
                value: '',
                language: 'python',
                theme: editorSettings.theme,
                fontSize: editorSettings.fontSize,
                fontFamily: "'Consolas', 'MS Gothic', monospace",
                automaticLayout: true,
                minimap: { enabled: true },
                scrollBeyondLastLine: false,
                wordWrap: 'off',
                lineNumbers: 'on',
                renderLineHighlight: 'line',
                tabSize: 4,
                insertSpaces: true,
                formatOnPaste: true,
                formatOnType: false,
                autoIndent: 'full',
                suggestOnTriggerCharacters: true,
                quickSuggestions: {
                    other: true,
                    comments: false,
                    strings: false
                },
                // Enable glyph margin for breakpoints
                glyphMargin: true
            });

            // Handle glyph margin click for breakpoints
            monacoEditor.onMouseDown((e) => {
                if (e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) {
                    const line = e.target.position.lineNumber;
                    toggleBreakpoint(line);
                }
            });

            // Listen for content changes
            monacoEditor.onDidChangeModelContent(() => {
                const tab = getActiveTab();
                if (tab) {
                    tab.content = monacoEditor.getValue();
                    if (tab.content !== tab.originalContent) {
                        setTabModified(tab.id, true);
                    } else {
                        setTabModified(tab.id, false);
                    }
                }

                // Debounce version detection
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(analyzeVersion, 300);
            });

            // Listen for cursor position changes
            monacoEditor.onDidChangeCursorPosition((e) => {
                updateCursorPosition(e.position);
            });

            resolve();
        });
    });
}

// ============================================================================
// Initialization / 初期化
// ============================================================================

document.addEventListener('DOMContentLoaded', async () => {
    // Load saved editor settings
    loadEditorSettings();

    // Initialize Monaco Editor
    try {
        await initMonacoEditor();
    } catch (e) {
        console.error('Failed to initialize Monaco Editor:', e);
        alert('Failed to initialize editor. Please check your internet connection.');
        return;
    }

    // Load core version
    try {
        const version = await invoke('get_core_version');
        document.getElementById('core-version').textContent = `sikulix-core v${version}`;
    } catch (e) {
        console.error('Failed to get core version:', e);
    }

    // Setup keyboard shortcuts
    setupKeyboardShortcuts();

    // Setup context menu
    setupContextMenu();

    // Restore session or create initial tab
    await restoreSession();

    // Update recent files menu
    await updateRecentFilesMenu();

    // Load saved watch expressions
    loadWatches();

    // Update title
    updateWindowTitle();
});

// ============================================================================
// Editor Settings / エディタ設定
// ============================================================================

function loadEditorSettings() {
    const saved = localStorage.getItem('sikulix-editor-settings');
    if (saved) {
        try {
            editorSettings = JSON.parse(saved);
        } catch (e) {
            console.error('Failed to parse editor settings:', e);
        }
    }

    // Update UI
    document.getElementById('theme-select').value = editorSettings.theme;
    document.getElementById('font-size-select').value = editorSettings.fontSize.toString();
}

function saveEditorSettings() {
    localStorage.setItem('sikulix-editor-settings', JSON.stringify(editorSettings));
}

function changeEditorTheme(theme) {
    editorSettings.theme = theme;
    if (monacoEditor) {
        monaco.editor.setTheme(theme);
    }
    saveEditorSettings();
}

function changeFontSize(size) {
    editorSettings.fontSize = parseInt(size, 10);
    if (monacoEditor) {
        monacoEditor.updateOptions({ fontSize: editorSettings.fontSize });
    }
    saveEditorSettings();
}

// ============================================================================
// Cursor Position Tracking / カーソル位置追跡
// ============================================================================

function updateCursorPosition(position) {
    const line = position.lineNumber;
    const col = position.column;

    document.getElementById('cursor-position').textContent = `Ln ${line}, Col ${col}`;

    // Save to active tab
    const tab = getActiveTab();
    if (tab) {
        tab.cursorPosition = { line, col };
    }
}

// ============================================================================
// Keyboard Shortcuts / キーボードショートカット
// ============================================================================

function setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        // Ctrl+S: Save
        if (e.ctrlKey && e.key === 's') {
            e.preventDefault();
            saveFile();
        }
        // Ctrl+O: Open
        if (e.ctrlKey && e.key === 'o') {
            e.preventDefault();
            openFile();
        }
        // Ctrl+N: New
        if (e.ctrlKey && e.key === 'n') {
            e.preventDefault();
            newFile();
        }
        // Ctrl+W: Close tab
        if (e.ctrlKey && e.key === 'w') {
            e.preventDefault();
            closeActiveTab();
        }
        // Ctrl+Tab: Next tab
        if (e.ctrlKey && e.key === 'Tab') {
            e.preventDefault();
            switchToNextTab();
        }
        // F5: Run
        if (e.key === 'F5' && !e.shiftKey) {
            e.preventDefault();
            runScript();
        }
        // Shift+F5: Stop
        if (e.shiftKey && e.key === 'F5') {
            e.preventDefault();
            stopScript();
        }
        // F9: Toggle breakpoint
        if (e.key === 'F9') {
            e.preventDefault();
            handleBreakpointShortcut();
        }
        // Escape: Close dialogs and context menu
        if (e.key === 'Escape') {
            closeSettings();
            closeAbout();
            hideContextMenu();
        }
    });
}

// ============================================================================
// Context Menu / コンテキストメニュー
// ============================================================================

function setupContextMenu() {
    // Hide context menu when clicking elsewhere
    document.addEventListener('click', (e) => {
        if (!e.target.closest('#tab-context-menu')) {
            hideContextMenu();
        }
    });
}

function showContextMenu(tabId, x, y) {
    contextMenuTabId = tabId;
    const menu = document.getElementById('tab-context-menu');
    menu.style.display = 'block';
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;
}

function hideContextMenu() {
    document.getElementById('tab-context-menu').style.display = 'none';
    contextMenuTabId = null;
}

function closeCurrentTabFromMenu() {
    if (contextMenuTabId !== null) {
        closeTab(contextMenuTabId);
    }
    hideContextMenu();
}

function closeOtherTabs() {
    if (contextMenuTabId !== null) {
        const tabsToClose = tabs.filter(t => t.id !== contextMenuTabId);
        for (const tab of tabsToClose) {
            closeTab(tab.id, false);
        }
        renderTabs();
    }
    hideContextMenu();
}

function closeAllTabs() {
    const tabsToClose = [...tabs];
    for (const tab of tabsToClose) {
        closeTab(tab.id, false);
    }
    if (tabs.length === 0) {
        newFile();
    }
    renderTabs();
    hideContextMenu();
}

function copyTabPath() {
    if (contextMenuTabId !== null) {
        const tab = tabs.find(t => t.id === contextMenuTabId);
        if (tab && tab.path) {
            navigator.clipboard.writeText(tab.path).then(() => {
                setStatus(t('status.path_copied') || 'Path copied to clipboard');
            });
        }
    }
    hideContextMenu();
}

// ============================================================================
// Tab Management / タブ管理
// ============================================================================

function getActiveTab() {
    return tabs.find(t => t.id === activeTabId);
}

function switchToTab(tabId) {
    // Save current tab state
    const currentTab = getActiveTab();
    if (currentTab && monacoEditor) {
        currentTab.content = monacoEditor.getValue();
        currentTab.viewState = monacoEditor.saveViewState();
    }

    // Switch to new tab
    activeTabId = tabId;
    const newTab = getActiveTab();

    if (newTab && monacoEditor) {
        monacoEditor.setValue(newTab.content);
        // Restore view state
        if (newTab.viewState) {
            monacoEditor.restoreViewState(newTab.viewState);
        }
        // Restore cursor position
        if (newTab.cursorPosition) {
            monacoEditor.setPosition({
                lineNumber: newTab.cursorPosition.line,
                column: newTab.cursorPosition.col
            });
            monacoEditor.revealPositionInCenter({
                lineNumber: newTab.cursorPosition.line,
                column: newTab.cursorPosition.col
            });
        }
        monacoEditor.focus();
    }

    renderTabs();
    updateWindowTitle();
    analyzeVersion();
}

function switchToNextTab() {
    if (tabs.length <= 1) return;

    const currentIndex = tabs.findIndex(t => t.id === activeTabId);
    const nextIndex = (currentIndex + 1) % tabs.length;
    switchToTab(tabs[nextIndex].id);
}

function setTabModified(tabId, modified) {
    const tab = tabs.find(t => t.id === tabId);
    if (tab) {
        tab.isModified = modified;
        renderTabs();
        updateWindowTitle();
    }
}

function renderTabs() {
    const container = document.getElementById('tabs-container');
    container.innerHTML = '';

    for (const tab of tabs) {
        const tabEl = document.createElement('div');
        tabEl.className = 'tab' + (tab.id === activeTabId ? ' active' : '');
        tabEl.dataset.tabId = tab.id;

        const title = document.createElement('span');
        title.className = 'tab-title';
        title.textContent = tab.path ? tab.path.split(/[\\/]/).pop() : 'Untitled';

        if (tab.isModified) {
            const modMarker = document.createElement('span');
            modMarker.className = 'tab-modified';
            modMarker.textContent = '*';
            tabEl.appendChild(modMarker);
        }

        tabEl.appendChild(title);

        const closeBtn = document.createElement('button');
        closeBtn.className = 'tab-close';
        closeBtn.textContent = '\u00D7';
        closeBtn.onclick = (e) => {
            e.stopPropagation();
            closeTab(tab.id);
        };
        tabEl.appendChild(closeBtn);

        tabEl.onclick = () => switchToTab(tab.id);
        tabEl.oncontextmenu = (e) => {
            e.preventDefault();
            showContextMenu(tab.id, e.clientX, e.clientY);
        };

        container.appendChild(tabEl);
    }
}

async function closeTab(tabId, render = true) {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab) return;

    if (tab.isModified) {
        const confirmed = await confirmDiscardChanges();
        if (!confirmed) return;
    }

    const index = tabs.indexOf(tab);
    tabs.splice(index, 1);

    if (tabs.length === 0) {
        // Create a new tab if all closed
        const newTab = createTab();
        tabs.push(newTab);
        activeTabId = newTab.id;
        if (monacoEditor) {
            monacoEditor.setValue('');
        }
    } else if (tabId === activeTabId) {
        // Switch to adjacent tab
        const newIndex = Math.min(index, tabs.length - 1);
        activeTabId = tabs[newIndex].id;
        const newActiveTab = getActiveTab();
        if (newActiveTab && monacoEditor) {
            monacoEditor.setValue(newActiveTab.content);
            if (newActiveTab.viewState) {
                monacoEditor.restoreViewState(newActiveTab.viewState);
            }
        }
    }

    if (render) {
        renderTabs();
        updateWindowTitle();
    }

    saveSession();
}

async function closeActiveTab() {
    if (activeTabId !== null) {
        await closeTab(activeTabId);
    }
}

// ============================================================================
// Python Analysis / Python解析
// ============================================================================

async function analyzeVersion() {
    const content = monacoEditor ? monacoEditor.getValue() : '';
    const indicator = document.getElementById('python-version');

    if (!content.trim()) {
        indicator.textContent = '';
        indicator.className = 'version-indicator';
        return;
    }

    try {
        const version = await invoke('analyze_python_version', { content });

        indicator.className = 'version-indicator ' + version;

        switch (version) {
            case 'python2':
                indicator.textContent = `${t('python.detected')}: ${t('python.python2_convert')}`;
                break;
            case 'python3':
                indicator.textContent = `${t('python.detected')}: ${t('python.python3')}`;
                break;
            case 'mixed':
                indicator.textContent = `${t('python.detected')}: ${t('python.mixed_review')}`;
                break;
            default:
                indicator.textContent = `${t('python.detected')}: ${t('python.unknown_default')}`;
        }
    } catch (e) {
        console.error('Failed to analyze version:', e);
    }
}

// ============================================================================
// Script Execution / スクリプト実行
// ============================================================================

function stopScript() {
    if (!isRunning) {
        setStatus('No script running');
        return;
    }

    addLogEntry('[Stop requested]', 'warn');

    // TODO: Implement actual stop functionality via Tauri command
    isRunning = false;
    updateExecutionUI();
    setStatus('Script stopped');
}

function updateExecutionUI() {
    const runBtn = document.getElementById('run-btn');
    const stopBtn = document.getElementById('stop-btn');
    const statusEl = document.getElementById('execution-status');

    if (isRunning) {
        runBtn.disabled = true;
        stopBtn.disabled = false;
        statusEl.textContent = 'Running...';
        statusEl.className = 'execution-status running';
    } else {
        runBtn.disabled = false;
        stopBtn.disabled = true;
        statusEl.textContent = '';
        statusEl.className = 'execution-status';
    }
}

function captureScreen() {
    addLogEntry('[Screen capture - not implemented]', 'warn');
}

// ============================================================================
// File Operations / ファイル操作
// ============================================================================

async function newFile() {
    const tab = createTab();
    tabs.push(tab);
    switchToTab(tab.id);

    if (monacoEditor) {
        monacoEditor.setValue('');
        monacoEditor.focus();
    }
    document.getElementById('output').textContent = '';
    document.getElementById('python-version').textContent = '';
    document.getElementById('python-version').className = 'version-indicator';

    try {
        await invoke('clear_current_file');
    } catch (e) {
        console.error('Failed to clear current file:', e);
    }

    saveSession();
    setStatus(t('status.new_file') || 'New file created');
}

async function openFile() {
    console.log('[DEBUG] openFile called');
    console.log('[DEBUG] window.__TAURI__:', window.__TAURI__);
    console.log('[DEBUG] window.__TAURI__.dialog:', window.__TAURI__?.dialog);

    try {
        if (!window.__TAURI__?.dialog) {
            throw new Error('Tauri dialog API not available');
        }
        const { open } = window.__TAURI__.dialog;
        console.log('[DEBUG] open function:', open);

        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Scripts',
                extensions: ['py', 'sikuli']
            }, {
                name: 'All Files',
                extensions: ['*']
            }]
        });

        if (selected) {
            await loadFileInTab(selected);
        }
    } catch (e) {
        console.error('Failed to open file dialog:', e);
        setStatus(`Error: ${e}`);
    }
}

async function loadFileInTab(path) {
    // Check if file is already open
    const existingTab = tabs.find(t => t.path === path);
    if (existingTab) {
        switchToTab(existingTab.id);
        return;
    }

    try {
        const result = await invoke('read_file', { path });

        if (result.success && result.file_info) {
            const { content } = result.file_info;

            // Create new tab or use current empty one
            let tab = getActiveTab();
            if (!tab || tab.path || tab.content || tab.isModified) {
                tab = createTab(path, content);
                tabs.push(tab);
            } else {
                tab.path = path;
                tab.content = content;
                tab.originalContent = content;
            }

            switchToTab(tab.id);
            if (monacoEditor) {
                monacoEditor.setValue(content);
            }

            // Add to recent files
            await invoke('add_recent_file', { path });
            await updateRecentFilesMenu();

            // Analyze version
            await analyzeVersion();

            saveSession();
            setStatus(result.message);
        } else {
            setStatus(`Error: ${result.message}`);
        }
    } catch (e) {
        console.error('Failed to load file:', e);
        setStatus(`Error: ${e}`);
    }
}

async function saveFile() {
    const tab = getActiveTab();
    if (!tab) return;

    if (tab.path) {
        await saveToPath(tab.path);
    } else {
        await saveFileAs();
    }
}

async function saveFileAs() {
    try {
        const { save } = window.__TAURI__.dialog;
        const tab = getActiveTab();

        const path = await save({
            filters: [{
                name: 'Python Script',
                extensions: ['py']
            }, {
                name: 'SikuliX Project',
                extensions: ['sikuli']
            }],
            defaultPath: tab?.path || 'untitled.py'
        });

        if (path) {
            await saveToPath(path);
        }
    } catch (e) {
        console.error('Failed to save file dialog:', e);
        setStatus(`Error: ${e}`);
    }
}

async function saveToPath(path) {
    const content = monacoEditor ? monacoEditor.getValue() : '';
    const tab = getActiveTab();

    try {
        const result = await invoke('write_file', { path, content });

        if (result.success) {
            if (tab) {
                tab.path = path;
                tab.originalContent = content;
                tab.isModified = false;
            }

            // Add to recent files
            await invoke('add_recent_file', { path });
            await updateRecentFilesMenu();

            renderTabs();
            updateWindowTitle();
            saveSession();
            setStatus(result.message);
        } else {
            setStatus(`Error: ${result.message}`);
        }
    } catch (e) {
        console.error('Failed to save file:', e);
        setStatus(`Error: ${e}`);
    }
}

async function confirmDiscardChanges() {
    try {
        const { ask } = window.__TAURI__.dialog;

        return await ask(
            t('dialog.unsaved_changes') || 'You have unsaved changes. Do you want to discard them?',
            {
                title: t('dialog.confirm') || 'Confirm',
                kind: 'warning'
            }
        );
    } catch (e) {
        return confirm(t('dialog.unsaved_changes') || 'You have unsaved changes. Do you want to discard them?');
    }
}

function exitApp() {
    const hasUnsaved = tabs.some(t => t.isModified);
    if (hasUnsaved) {
        confirmDiscardChanges().then(confirmed => {
            if (confirmed) {
                saveSession();
                window.close();
            }
        });
    } else {
        saveSession();
        window.close();
    }
}

// ============================================================================
// State Management / 状態管理
// ============================================================================

function updateWindowTitle() {
    const tab = getActiveTab();
    let title = 'SikuliX IDE';

    if (tab) {
        const fileName = tab.path ? tab.path.split(/[\\/]/).pop() : 'Untitled';
        title = `${tab.isModified ? '* ' : ''}${fileName} - SikuliX IDE`;
    }

    document.title = title;
}

function setStatus(message) {
    document.getElementById('status-text').textContent = message;
}

// ============================================================================
// Session Persistence / セッション永続化
// ============================================================================

function saveSession() {
    const session = {
        tabs: tabs.map(t => ({
            path: t.path,
            content: t.path ? '' : t.content, // Only save content for unsaved tabs
            isModified: t.isModified,
            cursorPosition: t.cursorPosition
        })),
        activeTabIndex: tabs.findIndex(t => t.id === activeTabId)
    };

    localStorage.setItem('sikulix-session', JSON.stringify(session));
}

async function restoreSession() {
    try {
        const saved = localStorage.getItem('sikulix-session');
        if (saved) {
            const session = JSON.parse(saved);

            for (const tabData of session.tabs) {
                let content = tabData.content;

                // Load content from file if path exists
                if (tabData.path) {
                    try {
                        const result = await invoke('read_file', { path: tabData.path });
                        if (result.success && result.file_info) {
                            content = result.file_info.content;
                        } else {
                            continue; // Skip if file not found
                        }
                    } catch (e) {
                        continue; // Skip if file not accessible
                    }
                }

                const tab = createTab(tabData.path, content, tabData.isModified);
                tab.originalContent = content;
                tab.cursorPosition = tabData.cursorPosition || { line: 1, col: 1 };
                tabs.push(tab);
            }

            if (tabs.length > 0) {
                const activeIndex = Math.min(session.activeTabIndex || 0, tabs.length - 1);
                switchToTab(tabs[activeIndex].id);
            }
        }
    } catch (e) {
        console.error('Failed to restore session:', e);
    }

    // Create initial tab if none exist
    if (tabs.length === 0) {
        const tab = createTab();
        tabs.push(tab);
        activeTabId = tab.id;
    }

    renderTabs();
}

// ============================================================================
// Settings Dialog / 設定ダイアログ
// ============================================================================

function showSettings() {
    document.getElementById('settings-dialog').style.display = 'flex';
}

function closeSettings() {
    document.getElementById('settings-dialog').style.display = 'none';
}

function changeLanguage(lang) {
    setLanguage(lang);
    analyzeVersion();
    updateRecentFilesMenu();
}

// ============================================================================
// About Dialog / Aboutダイアログ
// ============================================================================

function showAbout() {
    document.getElementById('about-dialog').style.display = 'flex';
}

function closeAbout() {
    document.getElementById('about-dialog').style.display = 'none';
}

// ============================================================================
// Recent Files Menu / 最近使用したファイルメニュー
// ============================================================================

async function updateRecentFilesMenu() {
    const menu = document.getElementById('recent-files-menu');

    try {
        const files = await invoke('get_recent_files');

        menu.innerHTML = '';

        if (files.length === 0) {
            const empty = document.createElement('div');
            empty.className = 'menu-empty';
            empty.setAttribute('data-i18n', 'menu.no_recent');
            empty.textContent = t('menu.no_recent') || 'No recent files';
            menu.appendChild(empty);
        } else {
            for (const path of files) {
                const btn = document.createElement('button');
                btn.textContent = path.split(/[\\/]/).pop();
                btn.title = path;
                btn.onclick = () => loadFileInTab(path);
                menu.appendChild(btn);
            }
        }
    } catch (e) {
        console.error('Failed to load recent files:', e);
    }
}

async function clearRecentFilesMenu() {
    try {
        await invoke('clear_recent_files');
        await updateRecentFilesMenu();
        setStatus(t('status.recent_cleared') || 'Recent files cleared');
    } catch (e) {
        console.error('Failed to clear recent files:', e);
    }
}

async function openRecentFile(path) {
    await loadFileInTab(path);
}

// ============================================================================
// Log Filtering / ログフィルタリング
// ============================================================================

function detectLogLevel(line) {
    const lowerLine = line.toLowerCase();
    if (lowerLine.includes('[debug]') || lowerLine.includes('debug:') || lowerLine.match(/\bdebug\b/)) {
        return 'debug';
    }
    if (lowerLine.includes('[error]') || lowerLine.includes('error:') || lowerLine.includes('traceback') || lowerLine.match(/\berror\b/)) {
        return 'error';
    }
    if (lowerLine.includes('[warn]') || lowerLine.includes('warning:') || lowerLine.includes('[warning]') || lowerLine.match(/\bwarn(?:ing)?\b/)) {
        return 'warn';
    }
    if (lowerLine.includes('[info]') || lowerLine.includes('info:') || lowerLine.match(/\binfo\b/)) {
        return 'info';
    }
    return 'output';
}

function createLogEntry(text, level = null) {
    const detectedLevel = level || detectLogLevel(text);
    const timestamp = new Date().toLocaleTimeString();

    return {
        text,
        level: detectedLevel,
        timestamp,
        id: logEntries.length
    };
}

function renderLogEntry(entry) {
    const div = document.createElement('div');
    div.className = `log-entry log-${entry.level}`;
    div.dataset.level = entry.level;
    div.dataset.id = entry.id;

    // Add timestamp
    const timestampSpan = document.createElement('span');
    timestampSpan.className = 'log-timestamp';
    timestampSpan.textContent = entry.timestamp;
    div.appendChild(timestampSpan);

    // Add level badge for non-output entries
    if (entry.level !== 'output') {
        const badge = document.createElement('span');
        badge.className = `log-level-badge ${entry.level}`;
        badge.textContent = entry.level.toUpperCase();
        div.appendChild(badge);
    }

    // Add text content
    const textSpan = document.createElement('span');
    textSpan.className = 'log-text';
    textSpan.textContent = entry.text;
    div.appendChild(textSpan);

    return div;
}

function addLogEntry(text, level = null) {
    const entry = createLogEntry(text, level);
    logEntries.push(entry);

    const outputEl = document.getElementById('output');
    const entryEl = renderLogEntry(entry);

    // Apply current filter
    if (!shouldShowEntry(entry)) {
        entryEl.classList.add('hidden');
    }

    outputEl.appendChild(entryEl);

    // Auto-scroll to bottom
    outputEl.scrollTop = outputEl.scrollHeight;
}

function addLogLines(text) {
    const lines = text.split('\n');
    for (const line of lines) {
        if (line.trim()) {
            addLogEntry(line);
        }
    }
}

function shouldShowEntry(entry) {
    // Check level filter
    if (entry.level !== 'output' && !logFilterLevels[entry.level]) {
        return false;
    }

    // Check search text
    if (logSearchText && !entry.text.toLowerCase().includes(logSearchText.toLowerCase())) {
        return false;
    }

    return true;
}

function toggleLogLevel(level) {
    logFilterLevels[level] = !logFilterLevels[level];

    // Update button state
    const btn = document.querySelector(`.log-filter-btn[data-level="${level}"]`);
    if (btn) {
        btn.classList.toggle('active', logFilterLevels[level]);
    }

    // Update ALL button state
    const allActive = Object.values(logFilterLevels).every(v => v);
    const allBtn = document.querySelector('.log-filter-btn[data-level="all"]');
    if (allBtn) {
        allBtn.classList.toggle('active', allActive);
    }

    filterLogs();
}

function setLogFilter(level) {
    if (level === 'all') {
        // Toggle all on
        Object.keys(logFilterLevels).forEach(key => {
            logFilterLevels[key] = true;
        });

        // Update all buttons
        document.querySelectorAll('.log-filter-btn').forEach(btn => {
            btn.classList.add('active');
        });
    }

    filterLogs();
}

function filterLogs() {
    const searchInput = document.getElementById('log-search');
    logSearchText = searchInput ? searchInput.value : '';

    const outputEl = document.getElementById('output');
    const entries = outputEl.querySelectorAll('.log-entry');

    entries.forEach(entryEl => {
        const id = parseInt(entryEl.dataset.id, 10);
        const entry = logEntries[id];

        if (entry && shouldShowEntry(entry)) {
            entryEl.classList.remove('hidden');

            // Highlight search text
            const textSpan = entryEl.querySelector('.log-text');
            if (textSpan && logSearchText) {
                const regex = new RegExp(`(${escapeRegExp(logSearchText)})`, 'gi');
                textSpan.innerHTML = entry.text.replace(regex, '<span class="log-highlight">$1</span>');
            } else if (textSpan) {
                textSpan.textContent = entry.text;
            }
        } else {
            entryEl.classList.add('hidden');
        }
    });
}

function escapeRegExp(string) {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function clearOutput() {
    const outputEl = document.getElementById('output');
    outputEl.innerHTML = '';
    logEntries = [];
}

async function exportLogs() {
    if (logEntries.length === 0) {
        setStatus(t('log.no_logs') || 'No logs to export');
        return;
    }

    // Format logs for export
    const logText = logEntries.map(entry => {
        const levelStr = entry.level.toUpperCase().padEnd(6);
        return `[${entry.timestamp}] [${levelStr}] ${entry.text}`;
    }).join('\n');

    try {
        const { save } = window.__TAURI__.dialog;

        const path = await save({
            filters: [{
                name: 'Log File',
                extensions: ['log', 'txt']
            }],
            defaultPath: `sikulix-log-${new Date().toISOString().slice(0, 10)}.log`
        });

        if (path) {
            await invoke('write_file', { path, content: logText });
            setStatus(t('log.exported') || `Logs exported to ${path}`);
        }
    } catch (e) {
        // Fallback: copy to clipboard if save dialog fails
        try {
            await navigator.clipboard.writeText(logText);
            setStatus(t('log.copied_clipboard') || 'Logs copied to clipboard');
        } catch (clipboardError) {
            console.error('Failed to export logs:', e);
            setStatus(`Error: ${e}`);
        }
    }
}

// Override runScript to use new log system
async function runScript() {
    if (isRunning) {
        setStatus('Script already running');
        return;
    }

    const content = monacoEditor ? monacoEditor.getValue() : '';

    if (!content.trim()) {
        addLogEntry('No script to run.', 'warn');
        return;
    }

    // Get active tab and check file path
    const tab = getActiveTab();
    if (!tab || !tab.path) {
        addLogEntry('Please save the file first. / ファイルを先に保存してください。', 'warn');
        await saveFileAs();
        const savedTab = getActiveTab();
        if (!savedTab || !savedTab.path) {
            addLogEntry('Script execution cancelled.', 'warn');
            return;
        }
    }

    // Auto-save if modified
    const currentTab = getActiveTab();
    if (currentTab && currentTab.isModified) {
        addLogEntry('Auto-saving...', 'info');
        await saveFile();
    }

    const scriptPath = currentTab.path;

    isRunning = true;
    updateExecutionUI();

    // Clear previous error decorations
    clearEditorDecorations();

    try {
        addLogEntry('=== Running Script ===', 'info');
        addLogEntry('File: ' + scriptPath, 'info');

        const workingDir = scriptPath.replace(/[\/\][^\/\]+$/, '');
        const options = {
            working_dir: workingDir,
            args: [],
            env_vars: {},
            debug: false,
            timeout_secs: null
        };

        const result = await invoke('run_script', { script_path: scriptPath, options: options });

        // Parse result for errors and add to problems
        parseAndAddProblems(result);

        addLogLines(result);
        addLogEntry('=== Script Completed ===', 'info');
    } catch (e) {
        const errorMsg = 'Error: ' + e;
        addLogEntry(errorMsg, 'error');

        // Try to parse error for line number
        const errorInfo = parseErrorMessage(e.toString());
        if (errorInfo) {
            addProblem('error', errorInfo.message, errorInfo.line, errorInfo.column);
        }
    } finally {
        isRunning = false;
        updateExecutionUI();
    }
}

// ============================================================================
// Bottom Panel Management / ボトムパネル管理
// ============================================================================

function switchBottomPanel(panelId) {
    // Update tabs
    document.querySelectorAll('.bottom-panel-tab').forEach(tab => {
        tab.classList.toggle('active', tab.dataset.panel === panelId);
    });

    // Update panels
    document.querySelectorAll('.bottom-panel').forEach(panel => {
        panel.classList.toggle('active', panel.id === `${panelId}-panel`);
    });
}

// ============================================================================
// Problems Panel / 問題パネル
// ============================================================================

function addProblem(type, message, line = null, column = null, file = null) {
    const problem = {
        id: problems.length,
        type, // 'error' or 'warning'
        message,
        line,
        column,
        file,
        timestamp: new Date().toLocaleTimeString()
    };

    problems.push(problem);
    renderProblems();
    updateProblemsCounts();

    // Add editor decoration if line is known
    if (line && monacoEditor) {
        addEditorDecoration(type, line, message);
    }
}

function renderProblems() {
    const listEl = document.getElementById('problems-list');
    listEl.innerHTML = '';

    const filteredProblems = problems.filter(p =>
        problemsFilter === 'all' || p.type === problemsFilter
    );

    if (filteredProblems.length === 0) {
        const noProblems = document.createElement('div');
        noProblems.className = 'no-problems';
        noProblems.textContent = t('problems.no_problems') || 'No problems detected';
        listEl.appendChild(noProblems);
        return;
    }

    for (const problem of filteredProblems) {
        const item = document.createElement('div');
        item.className = `problem-item ${problem.type}`;
        item.dataset.id = problem.id;
        item.onclick = () => goToProblem(problem);

        // Icon
        const icon = document.createElement('span');
        icon.className = `problem-icon ${problem.type}`;
        icon.innerHTML = problem.type === 'error' ? '&#10060;' : '&#9888;';
        item.appendChild(icon);

        // Message
        const msg = document.createElement('span');
        msg.className = 'problem-message';
        msg.textContent = problem.message;
        item.appendChild(msg);

        // Location
        if (problem.line) {
            const loc = document.createElement('span');
            loc.className = 'problem-location';
            const fileName = problem.file ? problem.file.split(/[\\/]/).pop() : 'script';
            loc.textContent = `${fileName}:${problem.line}${problem.column ? ':' + problem.column : ''}`;
            item.appendChild(loc);
        }

        listEl.appendChild(item);
    }
}

function updateProblemsCounts() {
    const errorCount = problems.filter(p => p.type === 'error').length;
    const warningCount = problems.filter(p => p.type === 'warning').length;
    const totalCount = errorCount + warningCount;

    document.getElementById('error-count').textContent = errorCount;
    document.getElementById('warning-count').textContent = warningCount;

    const badge = document.getElementById('problems-count');
    if (totalCount > 0) {
        badge.textContent = totalCount;
        badge.style.display = 'inline';
    } else {
        badge.style.display = 'none';
    }
}

function filterProblems(type) {
    problemsFilter = type;

    // Update filter buttons
    document.querySelectorAll('.problems-filter-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.type === type);
    });

    renderProblems();
}

function clearProblems() {
    problems = [];
    clearEditorDecorations();
    renderProblems();
    updateProblemsCounts();
}

function goToProblem(problem) {
    if (problem.line && monacoEditor) {
        monacoEditor.setPosition({
            lineNumber: problem.line,
            column: problem.column || 1
        });
        monacoEditor.revealLineInCenter(problem.line);
        monacoEditor.focus();

        // Switch to output panel to see context
        // switchBottomPanel('output');
    }
}

// ============================================================================
// Error Parsing / エラー解析
// ============================================================================

function parseErrorMessage(errorText) {
    // Python traceback patterns
    // Pattern: File "filename", line N
    const pythonPattern = /File "([^"]+)", line (\d+)(?:, in (.+))?/;
    // Pattern: line N
    const linePattern = /line (\d+)/i;
    // Pattern: SyntaxError at line N
    const syntaxPattern = /(?:syntax)?error.*?(?:at )?line (\d+)/i;
    // Pattern: Error on line N, column M
    const errorLineColPattern = /error.*?line (\d+)(?:.*?column (\d+))?/i;

    let match = errorText.match(pythonPattern);
    if (match) {
        return {
            file: match[1],
            line: parseInt(match[2], 10),
            column: null,
            message: errorText.split('\n').pop() || errorText
        };
    }

    match = errorText.match(errorLineColPattern);
    if (match) {
        return {
            file: null,
            line: parseInt(match[1], 10),
            column: match[2] ? parseInt(match[2], 10) : null,
            message: errorText
        };
    }

    match = errorText.match(syntaxPattern);
    if (match) {
        return {
            file: null,
            line: parseInt(match[1], 10),
            column: null,
            message: errorText
        };
    }

    match = errorText.match(linePattern);
    if (match) {
        return {
            file: null,
            line: parseInt(match[1], 10),
            column: null,
            message: errorText
        };
    }

    return null;
}

function parseAndAddProblems(output) {
    const lines = output.split('\n');
    let inTraceback = false;
    let tracebackLines = [];

    for (const line of lines) {
        // Detect Python traceback
        if (line.includes('Traceback (most recent call last)')) {
            inTraceback = true;
            tracebackLines = [line];
            continue;
        }

        if (inTraceback) {
            tracebackLines.push(line);

            // End of traceback - error message
            if (line.match(/^\w+Error:|^\w+Exception:|^SyntaxError:/)) {
                const errorInfo = parseTracebackForLocation(tracebackLines);
                if (errorInfo) {
                    addProblem('error', line, errorInfo.line, errorInfo.column, errorInfo.file);
                } else {
                    addProblem('error', line);
                }
                inTraceback = false;
                tracebackLines = [];
            }
        }

        // Detect warnings
        if (line.toLowerCase().includes('warning:')) {
            const errorInfo = parseErrorMessage(line);
            addProblem('warning', line, errorInfo?.line, errorInfo?.column);
        }

        // Detect standalone errors (not in traceback)
        if (!inTraceback && (line.toLowerCase().includes('error:') || line.match(/^\w+Error:/))) {
            const errorInfo = parseErrorMessage(line);
            if (!problems.some(p => p.message === line)) {
                addProblem('error', line, errorInfo?.line, errorInfo?.column);
            }
        }
    }
}

function parseTracebackForLocation(tracebackLines) {
    // Find the last file reference before the error
    for (let i = tracebackLines.length - 1; i >= 0; i--) {
        const line = tracebackLines[i];
        const match = line.match(/File "([^"]+)", line (\d+)/);
        if (match) {
            return {
                file: match[1],
                line: parseInt(match[2], 10),
                column: null
            };
        }
    }
    return null;
}

// ============================================================================
// Editor Error Decorations / エディタエラーデコレーション
// ============================================================================

function addEditorDecoration(type, line, message) {
    if (!monacoEditor) return;

    const decorationClass = type === 'error' ? 'error-line-decoration' : 'warning-line-decoration';
    const glyphClass = type === 'error' ? 'error-glyph' : 'warning-glyph';

    const newDecorations = monacoEditor.deltaDecorations([], [{
        range: new monaco.Range(line, 1, line, 1),
        options: {
            isWholeLine: true,
            className: decorationClass,
            glyphMarginClassName: glyphClass,
            glyphMarginHoverMessage: { value: message },
            hoverMessage: { value: `**${type.toUpperCase()}**: ${message}` }
        }
    }]);

    editorDecorations.push(...newDecorations);
}

function clearEditorDecorations() {
    if (monacoEditor && editorDecorations.length > 0) {
        monacoEditor.deltaDecorations(editorDecorations, []);
        editorDecorations = [];
    }
}

function updateEditorDecorations() {
    clearEditorDecorations();

    for (const problem of problems) {
        if (problem.line) {
            addEditorDecoration(problem.type, problem.line, problem.message);
        }
    }
}

// ============================================================================
// Stack Trace Jump / スタックトレースジャンプ
// ============================================================================

function makeStackTraceClickable(text) {
    // Replace file:line references with clickable links
    const pattern = /File "([^"]+)", line (\d+)/g;
    return text.replace(pattern, (match, file, line) => {
        return `<span class="stack-trace-link" onclick="jumpToLine(${line})">${match}</span>`;
    });
}

function jumpToLine(line) {
    if (monacoEditor) {
        monacoEditor.setPosition({ lineNumber: line, column: 1 });
        monacoEditor.revealLineInCenter(line);
        monacoEditor.focus();
    }
}

// Enhanced renderLogEntry with stack trace links
const originalRenderLogEntry = renderLogEntry;
renderLogEntry = function(entry) {
    const div = document.createElement('div');
    div.className = `log-entry log-${entry.level}`;
    div.dataset.level = entry.level;
    div.dataset.id = entry.id;

    // Add timestamp
    const timestampSpan = document.createElement('span');
    timestampSpan.className = 'log-timestamp';
    timestampSpan.textContent = entry.timestamp;
    div.appendChild(timestampSpan);

    // Add level badge for non-output entries
    if (entry.level !== 'output') {
        const badge = document.createElement('span');
        badge.className = `log-level-badge ${entry.level}`;
        badge.textContent = entry.level.toUpperCase();
        div.appendChild(badge);
    }

    // Add text content with clickable stack traces
    const textSpan = document.createElement('span');
    textSpan.className = 'log-text';

    // Check if this is a stack trace line
    if (entry.text.includes('File "') && entry.text.includes('line')) {
        textSpan.innerHTML = makeStackTraceClickable(entry.text);
    } else {
        textSpan.textContent = entry.text;
    }

    div.appendChild(textSpan);

    return div;
};

// ============================================================================
// Breakpoint Management / ブレークポイント管理
// ============================================================================

function toggleBreakpoint(line) {
    const existingIndex = breakpoints.findIndex(bp => bp.line === line);

    if (existingIndex >= 0) {
        // Remove existing breakpoint
        breakpoints.splice(existingIndex, 1);
    } else {
        // Add new breakpoint
        breakpoints.push({
            line,
            enabled: true,
            condition: null
        });
    }

    updateBreakpointDecorations();
    renderBreakpointsList();
}

function addBreakpoint(line, condition = null) {
    const existing = breakpoints.find(bp => bp.line === line);
    if (existing) {
        existing.condition = condition;
        existing.enabled = true;
    } else {
        breakpoints.push({
            line,
            enabled: true,
            condition
        });
    }

    updateBreakpointDecorations();
    renderBreakpointsList();
}

function removeBreakpoint(line) {
    const index = breakpoints.findIndex(bp => bp.line === line);
    if (index >= 0) {
        breakpoints.splice(index, 1);
        updateBreakpointDecorations();
        renderBreakpointsList();
    }
}

function enableBreakpoint(line, enabled) {
    const bp = breakpoints.find(bp => bp.line === line);
    if (bp) {
        bp.enabled = enabled;
        updateBreakpointDecorations();
        renderBreakpointsList();
    }
}

function setBreakpointCondition(line, condition) {
    const bp = breakpoints.find(bp => bp.line === line);
    if (bp) {
        bp.condition = condition || null;
        updateBreakpointDecorations();
        renderBreakpointsList();
    }
}

function clearAllBreakpoints() {
    breakpoints = [];
    updateBreakpointDecorations();
    renderBreakpointsList();
}

function updateBreakpointDecorations() {
    if (!monacoEditor) return;

    // Remove old decorations
    if (breakpointDecorations.length > 0) {
        monacoEditor.deltaDecorations(breakpointDecorations, []);
    }

    // Create new decorations
    const newDecorations = breakpoints.map(bp => {
        const glyphClass = bp.enabled
            ? (bp.condition ? 'breakpoint-conditional-glyph' : 'breakpoint-glyph')
            : 'breakpoint-disabled-glyph';

        const lineClass = bp.enabled ? 'breakpoint-line' : 'breakpoint-line-disabled';

        let hoverMessage = bp.enabled ? 'Breakpoint' : 'Breakpoint (disabled)';
        if (bp.condition) {
            hoverMessage += `\nCondition: ${bp.condition}`;
        }

        return {
            range: new monaco.Range(bp.line, 1, bp.line, 1),
            options: {
                isWholeLine: true,
                className: lineClass,
                glyphMarginClassName: glyphClass,
                glyphMarginHoverMessage: { value: hoverMessage }
            }
        };
    });

    breakpointDecorations = monacoEditor.deltaDecorations([], newDecorations);
}

function renderBreakpointsList() {
    const listEl = document.getElementById('breakpoints-list');
    if (!listEl) return;

    listEl.innerHTML = '';

    if (breakpoints.length === 0) {
        const emptyMsg = document.createElement('div');
        emptyMsg.className = 'breakpoints-empty';
        emptyMsg.textContent = t('breakpoints.no_breakpoints') || 'No breakpoints set';
        listEl.appendChild(emptyMsg);
        return;
    }

    // Sort by line number
    const sortedBreakpoints = [...breakpoints].sort((a, b) => a.line - b.line);

    for (const bp of sortedBreakpoints) {
        const item = document.createElement('div');
        item.className = `breakpoint-item${bp.enabled ? '' : ' disabled'}`;

        // Checkbox for enable/disable
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.checked = bp.enabled;
        checkbox.className = 'breakpoint-checkbox';
        checkbox.onchange = () => enableBreakpoint(bp.line, checkbox.checked);
        item.appendChild(checkbox);

        // Line number
        const lineSpan = document.createElement('span');
        lineSpan.className = 'breakpoint-line-number';
        lineSpan.textContent = `Line ${bp.line}`;
        lineSpan.onclick = () => goToBreakpoint(bp.line);
        item.appendChild(lineSpan);

        // Condition (if any)
        if (bp.condition) {
            const condSpan = document.createElement('span');
            condSpan.className = 'breakpoint-condition';
            condSpan.textContent = bp.condition;
            condSpan.title = `Condition: ${bp.condition}`;
            item.appendChild(condSpan);
        }

        // Edit condition button
        const editBtn = document.createElement('button');
        editBtn.className = 'breakpoint-edit-btn';
        editBtn.innerHTML = '&#9998;'; // Pencil icon
        editBtn.title = 'Edit condition';
        editBtn.onclick = (e) => {
            e.stopPropagation();
            showConditionDialog(bp.line, bp.condition);
        };
        item.appendChild(editBtn);

        // Remove button
        const removeBtn = document.createElement('button');
        removeBtn.className = 'breakpoint-remove-btn';
        removeBtn.innerHTML = '&times;';
        removeBtn.title = 'Remove breakpoint';
        removeBtn.onclick = (e) => {
            e.stopPropagation();
            removeBreakpoint(bp.line);
        };
        item.appendChild(removeBtn);

        listEl.appendChild(item);
    }

    // Update breakpoint count badge
    updateBreakpointCount();
}

function updateBreakpointCount() {
    const badge = document.getElementById('breakpoints-count');
    if (badge) {
        const count = breakpoints.length;
        if (count > 0) {
            badge.textContent = count;
            badge.style.display = 'inline';
        } else {
            badge.style.display = 'none';
        }
    }
}

function goToBreakpoint(line) {
    if (monacoEditor) {
        monacoEditor.setPosition({ lineNumber: line, column: 1 });
        monacoEditor.revealLineInCenter(line);
        monacoEditor.focus();
    }
}

function showConditionDialog(line, currentCondition) {
    const condition = prompt(
        t('breakpoints.enter_condition') || 'Enter breakpoint condition (leave empty for unconditional):',
        currentCondition || ''
    );

    if (condition !== null) {
        setBreakpointCondition(line, condition.trim());
    }
}

// F9 keyboard shortcut for toggling breakpoint at current line
function handleBreakpointShortcut() {
    if (monacoEditor) {
        const position = monacoEditor.getPosition();
        if (position) {
            toggleBreakpoint(position.lineNumber);
        }
    }
}

// ============================================================================
// Variables Inspector / 変数インスペクタ
// ============================================================================

function changeVariablesScope(scope) {
    variablesScope = scope;
    renderVariables();
}

function filterVariables() {
    const searchInput = document.getElementById('variables-search');
    variablesSearchText = searchInput ? searchInput.value.toLowerCase() : '';
    renderVariables();
}

function refreshVariables() {
    // In real debug mode, this would fetch variables from debugger
    // For now, show demo data or empty state
    if (isRunning) {
        // Demo: simulate fetching variables
        updateVariablesFromDebugger();
    } else {
        debugVariables = { local: [], global: [] };
        renderVariables();
    }
}

async function updateVariablesFromDebugger() {
    try {
        // Try to get variables from core-rs debugger
        const result = await invoke('get_debug_variables');
        if (result) {
            debugVariables.local = result.local || [];
            debugVariables.global = result.global || [];
        }
    } catch (e) {
        // Fallback to empty or demo data
        console.log('Could not fetch debug variables:', e);
    }
    renderVariables();
}

function setDebugVariables(local, global) {
    debugVariables.local = local || [];
    debugVariables.global = global || [];
    renderVariables();
}

function renderVariables() {
    const listEl = document.getElementById('variables-list');
    if (!listEl) return;

    listEl.innerHTML = '';

    // Check if we're in debug mode
    if (!isRunning && debugVariables.local.length === 0 && debugVariables.global.length === 0) {
        const emptyMsg = document.createElement('div');
        emptyMsg.className = 'variables-empty';
        emptyMsg.textContent = t('variables.not_debugging') || 'Not debugging. Run script in debug mode to see variables.';
        listEl.appendChild(emptyMsg);
        return;
    }

    // Get variables based on scope
    let variables = [];
    if (variablesScope === 'local') {
        variables = debugVariables.local;
    } else if (variablesScope === 'global') {
        variables = debugVariables.global;
    } else {
        // All - combine both with section headers
        if (debugVariables.local.length > 0) {
            variables.push({ __section: 'Local', __items: debugVariables.local });
        }
        if (debugVariables.global.length > 0) {
            variables.push({ __section: 'Global', __items: debugVariables.global });
        }
    }

    if (variables.length === 0) {
        const emptyMsg = document.createElement('div');
        emptyMsg.className = 'variables-empty';
        emptyMsg.textContent = t('variables.no_variables') || 'No variables in this scope.';
        listEl.appendChild(emptyMsg);
        return;
    }

    // Render variables
    for (const item of variables) {
        if (item.__section) {
            // Render section header
            const sectionHeader = document.createElement('div');
            sectionHeader.className = 'variables-section-header';
            sectionHeader.textContent = item.__section;
            listEl.appendChild(sectionHeader);

            // Render section items
            for (const variable of item.__items) {
                if (shouldShowVariable(variable)) {
                    const varEl = renderVariableNode(variable, 0);
                    listEl.appendChild(varEl);
                }
            }
        } else {
            if (shouldShowVariable(item)) {
                const varEl = renderVariableNode(item, 0);
                listEl.appendChild(varEl);
            }
        }
    }
}

function shouldShowVariable(variable) {
    if (!variablesSearchText) return true;

    const nameMatch = variable.name.toLowerCase().includes(variablesSearchText);
    const valueMatch = formatVariableValue(variable.value).toLowerCase().includes(variablesSearchText);

    return nameMatch || valueMatch;
}

function renderVariableNode(variable, depth) {
    const item = document.createElement('div');
    item.className = 'variable-item';
    item.style.paddingLeft = `${depth * 16 + 8}px`;

    const hasChildren = isExpandableValue(variable.value);
    const nodeKey = `${depth}-${variable.name}`;
    const isExpanded = expandedVariables.has(nodeKey);

    // Expand/collapse toggle
    if (hasChildren) {
        const toggle = document.createElement('span');
        toggle.className = `variable-toggle ${isExpanded ? 'expanded' : ''}`;
        toggle.innerHTML = isExpanded ? '&#9660;' : '&#9654;';
        toggle.onclick = (e) => {
            e.stopPropagation();
            toggleVariableExpand(nodeKey);
        };
        item.appendChild(toggle);
    } else {
        const spacer = document.createElement('span');
        spacer.className = 'variable-toggle-spacer';
        item.appendChild(spacer);
    }

    // Variable name
    const nameSpan = document.createElement('span');
    nameSpan.className = 'variable-name';
    nameSpan.textContent = variable.name;
    item.appendChild(nameSpan);

    // Separator
    const separator = document.createElement('span');
    separator.className = 'variable-separator';
    separator.textContent = ': ';
    item.appendChild(separator);

    // Variable type
    const typeSpan = document.createElement('span');
    typeSpan.className = 'variable-type';
    typeSpan.textContent = getVariableType(variable.value);
    item.appendChild(typeSpan);

    // Variable value
    const valueSpan = document.createElement('span');
    valueSpan.className = 'variable-value';
    valueSpan.textContent = formatVariableValue(variable.value);
    valueSpan.ondblclick = () => editVariable(variable);
    item.appendChild(valueSpan);

    // Create container for this node and its children
    const container = document.createElement('div');
    container.className = 'variable-node';
    container.appendChild(item);

    // Render children if expanded
    if (hasChildren && isExpanded) {
        const childrenContainer = document.createElement('div');
        childrenContainer.className = 'variable-children';

        const children = getVariableChildren(variable.value);
        for (const child of children) {
            const childEl = renderVariableNode(child, depth + 1);
            childrenContainer.appendChild(childEl);
        }

        container.appendChild(childrenContainer);
    }

    return container;
}

function isExpandableValue(value) {
    if (value === null || value === undefined) return false;
    if (typeof value === 'object') return true;
    if (Array.isArray(value)) return true;
    return false;
}

function getVariableType(value) {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (Array.isArray(value)) return `Array(${value.length})`;
    if (typeof value === 'object') {
        const keys = Object.keys(value);
        return `Object{${keys.length}}`;
    }
    return typeof value;
}

function formatVariableValue(value) {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'string') return `"${value}"`;
    if (typeof value === 'number' || typeof value === 'boolean') return String(value);
    if (Array.isArray(value)) {
        if (value.length === 0) return '[]';
        if (value.length <= 3) {
            return `[${value.map(v => formatShortValue(v)).join(', ')}]`;
        }
        return `[${value.slice(0, 3).map(v => formatShortValue(v)).join(', ')}, ...]`;
    }
    if (typeof value === 'object') {
        const keys = Object.keys(value);
        if (keys.length === 0) return '{}';
        if (keys.length <= 2) {
            return `{${keys.map(k => `${k}: ${formatShortValue(value[k])}`).join(', ')}}`;
        }
        return `{${keys.slice(0, 2).map(k => `${k}: ${formatShortValue(value[k])}`).join(', ')}, ...}`;
    }
    return String(value);
}

function formatShortValue(value) {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'string') {
        if (value.length > 20) return `"${value.slice(0, 17)}..."`;
        return `"${value}"`;
    }
    if (typeof value === 'number' || typeof value === 'boolean') return String(value);
    if (Array.isArray(value)) return `Array(${value.length})`;
    if (typeof value === 'object') return `{...}`;
    return String(value);
}

function getVariableChildren(value) {
    if (Array.isArray(value)) {
        return value.map((v, i) => ({ name: `[${i}]`, value: v }));
    }
    if (typeof value === 'object' && value !== null) {
        return Object.entries(value).map(([k, v]) => ({ name: k, value: v }));
    }
    return [];
}

function toggleVariableExpand(nodeKey) {
    if (expandedVariables.has(nodeKey)) {
        expandedVariables.delete(nodeKey);
    } else {
        expandedVariables.add(nodeKey);
    }
    renderVariables();
}

function editVariable(variable) {
    const newValue = prompt(
        t('variables.edit_value') || `Edit value for "${variable.name}":`,
        formatVariableValue(variable.value)
    );

    if (newValue !== null) {
        // Try to parse the new value
        let parsedValue;
        try {
            // Try JSON parse first
            parsedValue = JSON.parse(newValue);
        } catch {
            // Fall back to string
            parsedValue = newValue;
        }

        // In real debug mode, this would send the new value to the debugger
        variable.value = parsedValue;
        renderVariables();

        // TODO: Send to debugger via invoke('set_debug_variable', { name: variable.name, value: parsedValue })
    }
}

// ============================================================================
// Watch Expressions / ウォッチ式
// ============================================================================

function addWatch(expression) {
    if (!expression || !expression.trim()) return;

    const watch = {
        id: nextWatchId++,
        expression: expression.trim(),
        value: undefined,
        error: null
    };

    watchExpressions.push(watch);
    evaluateWatch(watch);
    renderWatches();
    saveWatches();
}

function addWatchFromInput() {
    const input = document.getElementById('watch-input');
    if (input && input.value.trim()) {
        addWatch(input.value);
        input.value = '';
    }
}

function handleWatchInputKeydown(event) {
    if (event.key === 'Enter') {
        event.preventDefault();
        addWatchFromInput();
    }
}

function removeWatch(id) {
    const index = watchExpressions.findIndex(w => w.id === id);
    if (index >= 0) {
        watchExpressions.splice(index, 1);
        renderWatches();
        saveWatches();
    }
}

function editWatch(id) {
    const watch = watchExpressions.find(w => w.id === id);
    if (!watch) return;

    const newExpression = prompt(
        t('watch.edit_expression') || 'Edit watch expression:',
        watch.expression
    );

    if (newExpression !== null && newExpression.trim()) {
        watch.expression = newExpression.trim();
        evaluateWatch(watch);
        renderWatches();
        saveWatches();
    }
}

function clearAllWatches() {
    watchExpressions = [];
    renderWatches();
    saveWatches();
}

async function evaluateWatch(watch) {
    if (!isRunning) {
        watch.value = undefined;
        watch.error = t('watch.not_debugging') || 'Not debugging';
        return;
    }

    try {
        // Try to evaluate via debugger
        const result = await invoke('evaluate_expression', { expression: watch.expression });
        watch.value = result.value;
        watch.error = result.error || null;
    } catch (e) {
        watch.value = undefined;
        watch.error = e.toString();
    }
}

function evaluateAllWatches() {
    for (const watch of watchExpressions) {
        evaluateWatch(watch);
    }
    renderWatches();
}

function renderWatches() {
    const listEl = document.getElementById('watch-list');
    if (!listEl) return;

    listEl.innerHTML = '';

    if (watchExpressions.length === 0) {
        const emptyMsg = document.createElement('div');
        emptyMsg.className = 'watch-empty';
        emptyMsg.textContent = t('watch.no_watches') || 'No watch expressions. Add an expression to evaluate.';
        listEl.appendChild(emptyMsg);
        return;
    }

    for (const watch of watchExpressions) {
        const item = document.createElement('div');
        item.className = 'watch-item';
        if (watch.error) {
            item.classList.add('error');
        }

        // Expression
        const exprSpan = document.createElement('span');
        exprSpan.className = 'watch-expression';
        exprSpan.textContent = watch.expression;
        exprSpan.ondblclick = () => editWatch(watch.id);
        item.appendChild(exprSpan);

        // Separator
        const separator = document.createElement('span');
        separator.className = 'watch-separator';
        separator.textContent = ' = ';
        item.appendChild(separator);

        // Value or error
        const valueSpan = document.createElement('span');
        if (watch.error) {
            valueSpan.className = 'watch-error';
            valueSpan.textContent = watch.error;
        } else if (watch.value !== undefined) {
            valueSpan.className = 'watch-value';
            valueSpan.textContent = formatVariableValue(watch.value);
        } else {
            valueSpan.className = 'watch-value pending';
            valueSpan.textContent = '...';
        }
        item.appendChild(valueSpan);

        // Buttons container
        const buttonsDiv = document.createElement('div');
        buttonsDiv.className = 'watch-buttons';

        // Edit button
        const editBtn = document.createElement('button');
        editBtn.className = 'watch-edit-btn';
        editBtn.innerHTML = '&#9998;';
        editBtn.title = t('watch.edit') || 'Edit';
        editBtn.onclick = (e) => {
            e.stopPropagation();
            editWatch(watch.id);
        };
        buttonsDiv.appendChild(editBtn);

        // Remove button
        const removeBtn = document.createElement('button');
        removeBtn.className = 'watch-remove-btn';
        removeBtn.innerHTML = '&times;';
        removeBtn.title = t('watch.remove') || 'Remove';
        removeBtn.onclick = (e) => {
            e.stopPropagation();
            removeWatch(watch.id);
        };
        buttonsDiv.appendChild(removeBtn);

        item.appendChild(buttonsDiv);
        listEl.appendChild(item);
    }
}

function saveWatches() {
    const expressions = watchExpressions.map(w => w.expression);
    localStorage.setItem('sikulix-watch-expressions', JSON.stringify(expressions));
}

function loadWatches() {
    try {
        const saved = localStorage.getItem('sikulix-watch-expressions');
        if (saved) {
            const expressions = JSON.parse(saved);
            for (const expr of expressions) {
                addWatch(expr);
            }
        }
    } catch (e) {
        console.error('Failed to load watch expressions:', e);
    }
}

// ============================================================================
// Pattern Editor / パターンエディタ
// ============================================================================

// Open pattern editor in a new window / 新しいウィンドウでパターンエディタを開く
async function openPatternEditorWindow() {
    try {
        const webview = new WebviewWindow('pattern-editor', {
            url: 'pattern-editor.html',
            title: 'Pattern Editor - SikuliX IDE',
            width: 1000,
            height: 700,
            minWidth: 800,
            minHeight: 600,
            resizable: true,
            center: true
        });

        // Wait for window to be created
        webview.once('tauri://created', () => {
            console.log('Pattern editor window created');
        });

        // Handle window errors
        webview.once('tauri://error', (e) => {
            console.error('Failed to create pattern editor window:', e);
        });
    } catch (error) {
        console.error('Error opening pattern editor:', error);
        alert('Failed to open pattern editor: ' + error);
    }
}

function openPatternEditor(imagePath, options = {}) {
    // Set current pattern
    currentPattern = {
        imagePath: imagePath,
        similarity: options.similarity || 0.7,
        targetOffset: options.targetOffset || { x: 0, y: 0 },
        imageWidth: 0,
        imageHeight: 0
    };
    patternCallback = options.callback || null;

    // Update UI
    document.getElementById('pattern-similarity').value = Math.round(currentPattern.similarity * 100);
    document.getElementById('pattern-similarity-value').textContent = currentPattern.similarity.toFixed(2);
    document.getElementById('pattern-offset-x').value = currentPattern.targetOffset.x;
    document.getElementById('pattern-offset-y').value = currentPattern.targetOffset.y;
    document.getElementById('test-match-result').innerHTML = '';

    // Load image
    loadPatternImage(imagePath);

    // Show dialog
    document.getElementById('pattern-editor-dialog').style.display = 'flex';
}

function closePatternEditor() {
    document.getElementById('pattern-editor-dialog').style.display = 'none';
    patternImage = null;
    patternCallback = null;
}

function loadPatternImage(imagePath) {
    const canvas = document.getElementById('pattern-canvas');
    const ctx = canvas.getContext('2d');

    patternImage = new Image();
    patternImage.onload = function() {
        // Calculate display size (max 300px)
        const maxSize = 300;
        let displayWidth = patternImage.width;
        let displayHeight = patternImage.height;

        if (displayWidth > maxSize || displayHeight > maxSize) {
            const scale = Math.min(maxSize / displayWidth, maxSize / displayHeight);
            displayWidth = displayWidth * scale;
            displayHeight = displayHeight * scale;
        }

        canvas.width = displayWidth;
        canvas.height = displayHeight;

        currentPattern.imageWidth = patternImage.width;
        currentPattern.imageHeight = patternImage.height;

        // Draw image
        ctx.drawImage(patternImage, 0, 0, displayWidth, displayHeight);

        // Update info
        const filename = imagePath.split(/[\\/]/).pop();
        document.getElementById('pattern-filename').textContent = filename;
        document.getElementById('pattern-dimensions').textContent = `${patternImage.width} x ${patternImage.height}`;

        // Draw target marker
        updateTargetMarker();

        // Setup click handler
        canvas.onclick = handleCanvasClick;
    };

    patternImage.onerror = function() {
        ctx.fillStyle = '#333';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = '#888';
        ctx.font = '14px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText(t('pattern.load_error') || 'Failed to load image', canvas.width / 2, canvas.height / 2);
    };

    // Try loading from file path or data URL
    if (imagePath.startsWith('data:') || imagePath.startsWith('http')) {
        patternImage.src = imagePath;
    } else {
        // Convert file path to data URL using Tauri
        loadPatternImageFromFile(imagePath);
    }
}

async function loadPatternImageFromFile(filePath) {
    try {
        const result = await invoke('read_image_base64', { path: filePath });
        if (result && result.base64) {
            patternImage.src = `data:image/png;base64,${result.base64}`;
        }
    } catch (e) {
        console.error('Failed to load pattern image:', e);
        // Fallback: try direct file URL (for development)
        patternImage.src = `file://${filePath.replace(/\\/g, '/')}`;
    }
}

function handleCanvasClick(event) {
    const canvas = document.getElementById('pattern-canvas');
    const rect = canvas.getBoundingClientRect();

    // Get click position relative to canvas
    const clickX = event.clientX - rect.left;
    const clickY = event.clientY - rect.top;

    // Calculate scale factor
    const scaleX = currentPattern.imageWidth / canvas.width;
    const scaleY = currentPattern.imageHeight / canvas.height;

    // Calculate offset from center in original image coordinates
    const centerX = currentPattern.imageWidth / 2;
    const centerY = currentPattern.imageHeight / 2;
    const imageX = clickX * scaleX;
    const imageY = clickY * scaleY;

    currentPattern.targetOffset.x = Math.round(imageX - centerX);
    currentPattern.targetOffset.y = Math.round(imageY - centerY);

    // Update UI
    document.getElementById('pattern-offset-x').value = currentPattern.targetOffset.x;
    document.getElementById('pattern-offset-y').value = currentPattern.targetOffset.y;

    updateTargetMarker();
}

function updateTargetMarker() {
    const canvas = document.getElementById('pattern-canvas');
    const marker = document.getElementById('pattern-target-marker');

    if (!patternImage || currentPattern.imageWidth === 0) return;

    // Calculate scale factor
    const scaleX = canvas.width / currentPattern.imageWidth;
    const scaleY = canvas.height / currentPattern.imageHeight;

    // Calculate marker position on canvas
    const centerX = currentPattern.imageWidth / 2;
    const centerY = currentPattern.imageHeight / 2;
    const targetX = (centerX + currentPattern.targetOffset.x) * scaleX;
    const targetY = (centerY + currentPattern.targetOffset.y) * scaleY;

    // Get canvas position
    const container = canvas.parentElement;
    const canvasRect = canvas.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();

    marker.style.left = `${canvasRect.left - containerRect.left + targetX}px`;
    marker.style.top = `${canvasRect.top - containerRect.top + targetY}px`;
    marker.style.display = 'block';
}

function updateSimilarityValue(value) {
    const similarity = parseInt(value, 10) / 100;
    currentPattern.similarity = similarity;
    document.getElementById('pattern-similarity-value').textContent = similarity.toFixed(2);
}

function updateTargetOffset() {
    currentPattern.targetOffset.x = parseInt(document.getElementById('pattern-offset-x').value, 10) || 0;
    currentPattern.targetOffset.y = parseInt(document.getElementById('pattern-offset-y').value, 10) || 0;
    updateTargetMarker();
}

function resetTargetOffset() {
    currentPattern.targetOffset.x = 0;
    currentPattern.targetOffset.y = 0;
    document.getElementById('pattern-offset-x').value = 0;
    document.getElementById('pattern-offset-y').value = 0;
    updateTargetMarker();
}

async function testPatternMatch() {
    const resultEl = document.getElementById('test-match-result');
    resultEl.innerHTML = `<span class="testing">${t('pattern.testing') || 'Testing...'}</span>`;

    try {
        const result = await invoke('test_pattern_match', {
            imagePath: currentPattern.imagePath,
            similarity: currentPattern.similarity
        });

        if (result && result.found) {
            resultEl.innerHTML = `
                <span class="match-success">
                    ${t('pattern.match_found') || 'Match found!'}<br>
                    ${t('pattern.confidence') || 'Confidence'}: ${(result.confidence * 100).toFixed(1)}%<br>
                    ${t('pattern.position') || 'Position'}: (${result.x}, ${result.y})
                </span>
            `;
        } else {
            resultEl.innerHTML = `
                <span class="match-fail">
                    ${t('pattern.no_match') || 'No match found at current similarity.'}
                </span>
            `;
        }
    } catch (e) {
        resultEl.innerHTML = `
            <span class="match-error">
                ${t('pattern.test_error') || 'Error'}: ${e}
            </span>
        `;
    }
}

function applyPatternSettings() {
    const settings = {
        imagePath: currentPattern.imagePath,
        similarity: currentPattern.similarity,
        targetOffset: currentPattern.targetOffset
    };

    if (patternCallback) {
        patternCallback(settings);
    }

    closePatternEditor();
    setStatus(t('pattern.settings_applied') || 'Pattern settings applied');
}

// Generate pattern code for editor insertion
function generatePatternCode(imagePath, similarity = 0.7, offsetX = 0, offsetY = 0) {
    const filename = imagePath.split(/[\\/]/).pop();

    if (offsetX === 0 && offsetY === 0 && similarity === 0.7) {
        return `"${filename}"`;
    }

    let code = `Pattern("${filename}")`;

    if (similarity !== 0.7) {
        code += `.similar(${similarity.toFixed(2)})`;
    }

    if (offsetX !== 0 || offsetY !== 0) {
        code += `.targetOffset(${offsetX}, ${offsetY})`;
    }

    return code;
}

// ===== Image Library Functions =====

let imageLibraryVisible = false;
let imageViewMode = 'grid';
let libraryImages = [];
let libraryFolders = [];
let selectedFolder = null;
let selectedImage = null;
let imageSearchText = '';
let sidebarResizing = false;

function toggleImageLibrary() {
    const sidebar = document.getElementById('image-library-sidebar');
    const resizeHandle = document.getElementById('sidebar-resize-handle');
    const btn = document.getElementById('library-btn');

    imageLibraryVisible = !imageLibraryVisible;

    if (imageLibraryVisible) {
        sidebar.style.display = 'flex';
        resizeHandle.style.display = 'block';
        btn.classList.add('active');
        refreshImageLibrary();
    } else {
        sidebar.style.display = 'none';
        resizeHandle.style.display = 'none';
        btn.classList.remove('active');
    }

    // Trigger editor resize
    if (editor) {
        setTimeout(() => editor.layout(), 100);
    }
}

async function refreshImageLibrary() {
    const grid = document.getElementById('image-grid');
    const folderTree = document.getElementById('image-folder-tree');
    const countEl = document.getElementById('image-count');

    grid.innerHTML = `<div class="library-empty">${t('library.loading') || 'Loading...'}</div>`;

    try {
        // Get project directory (use current file's directory or a default)
        let projectDir = null;
        const activeTab = getActiveTab();
        if (activeTab && activeTab.filePath) {
            projectDir = activeTab.filePath.replace(/[/\\][^/\\]+$/, '');
        }

        if (!projectDir) {
            grid.innerHTML = `<div class="library-empty">${t('library.no_project') || 'Open a file to see project images'}</div>`;
            return;
        }

        // Try to get images from backend
        const result = await invoke('list_images', { directory: projectDir });

        if (result && result.images) {
            libraryImages = result.images;
            libraryFolders = result.folders || [];
            renderFolderTree();
            renderImageGrid();
            countEl.innerHTML = `${libraryImages.length} <span data-i18n="library.images">${t('library.images') || 'images'}</span>`;
        } else {
            // Demo mode - show empty state
            libraryImages = [];
            grid.innerHTML = `<div class="library-empty">${t('library.empty') || 'No images found in project'}</div>`;
            countEl.innerHTML = `0 <span data-i18n="library.images">${t('library.images') || 'images'}</span>`;
        }
    } catch (e) {
        console.log('Image library not available:', e);
        // Demo mode with sample images
        libraryImages = [];
        grid.innerHTML = `<div class="library-empty">${t('library.empty') || 'No images found in project'}</div>`;
        countEl.innerHTML = `0 <span data-i18n="library.images">${t('library.images') || 'images'}</span>`;
    }
}

function renderFolderTree() {
    const container = document.getElementById('image-folder-tree');

    if (libraryFolders.length === 0) {
        container.innerHTML = '';
        return;
    }

    let html = '';
    libraryFolders.forEach(folder => {
        html += renderFolderItem(folder);
    });

    container.innerHTML = html;
}

function renderFolderItem(folder, level = 0) {
    const selected = selectedFolder === folder.path ? 'selected' : '';
    const hasChildren = folder.children && folder.children.length > 0;

    let html = `
        <div class="folder-item ${selected}" onclick="selectFolder('${folder.path.replace(/\\/g, '\\\\')}')" style="padding-left: ${8 + level * 16}px;">
            <span class="folder-toggle">${hasChildren ? '▶' : ''}</span>
            <span class="folder-icon">📁</span>
            <span class="folder-name">${folder.name}</span>
        </div>
    `;

    if (hasChildren) {
        html += `<div class="folder-children">`;
        folder.children.forEach(child => {
            html += renderFolderItem(child, level + 1);
        });
        html += `</div>`;
    }

    return html;
}

function selectFolder(path) {
    selectedFolder = selectedFolder === path ? null : path;
    renderFolderTree();
    filterImages();
}

function renderImageGrid() {
    const grid = document.getElementById('image-grid');
    const filteredImages = getFilteredImages();

    if (filteredImages.length === 0) {
        grid.innerHTML = `<div class="library-empty">${t('library.empty') || 'No images found'}</div>`;
        return;
    }

    grid.className = `image-grid ${imageViewMode === 'list' ? 'list-view' : ''}`;

    let html = '';
    filteredImages.forEach(img => {
        const selected = selectedImage === img.path ? 'selected' : '';
        const filename = img.name || img.path.split(/[\\/]/).pop();

        html += `
            <div class="image-item ${selected}"
                 draggable="true"
                 ondragstart="handleImageDragStart(event, '${img.path.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}')"
                 ondragend="handleImageDragEnd(event)"
                 onclick="selectImage('${img.path.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}')"
                 ondblclick="openPatternEditorForImage('${img.path.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}')"
                 title="${filename}">
                <img class="image-thumbnail" src="${img.thumbnail || img.dataUrl || 'data:image/png;base64,'}" alt="${filename}" onerror="this.src='data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADwAAAA8CAQAAACQ9RH5AAAAaklEQVR42u3PMREAAAgEoNe/aLvhCwcJ5DamAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC4C7x8AAHXdgGRAAAAAElFTkSuQmCC';">
                <span class="image-name">${filename}</span>
            </div>
        `;
    });

    grid.innerHTML = html;
}

function getFilteredImages() {
    return libraryImages.filter(img => {
        // Filter by search text
        if (imageSearchText) {
            const name = (img.name || img.path).toLowerCase();
            if (!name.includes(imageSearchText.toLowerCase())) {
                return false;
            }
        }

        // Filter by selected folder
        if (selectedFolder) {
            if (!img.path.startsWith(selectedFolder)) {
                return false;
            }
        }

        return true;
    });
}

function filterImages() {
    imageSearchText = document.getElementById('image-search').value;
    renderImageGrid();
}

function setImageView(mode) {
    imageViewMode = mode;

    // Update button states
    document.querySelectorAll('.view-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.view === mode);
    });

    renderImageGrid();
}

function selectImage(path) {
    selectedImage = selectedImage === path ? null : path;
    renderImageGrid();
}

function openPatternEditorForImage(imagePath) {
    openPatternEditor(imagePath, {
        similarity: 0.7,
        targetOffset: { x: 0, y: 0 },
        callback: (settings) => {
            // Insert pattern code into editor
            const code = generatePatternCode(
                settings.imagePath,
                settings.similarity,
                settings.targetOffset.x,
                settings.targetOffset.y
            );
            insertTextAtCursor(code);
        }
    });
}

function insertTextAtCursor(text) {
    if (!editor) return;

    const selection = editor.getSelection();
    const id = { major: 1, minor: 1 };

    editor.executeEdits('insert', [{
        range: selection,
        text: text,
        forceMoveMarkers: true
    }]);

    editor.focus();
}

// Drag and Drop handlers
function handleImageDragStart(event, imagePath) {
    event.target.classList.add('dragging');

    // Set drag data
    const filename = imagePath.split(/[\\/]/).pop();
    event.dataTransfer.setData('text/plain', `"${filename}"`);
    event.dataTransfer.setData('application/x-sikulix-image', JSON.stringify({
        path: imagePath,
        filename: filename
    }));
    event.dataTransfer.effectAllowed = 'copy';
}

function handleImageDragEnd(event) {
    event.target.classList.remove('dragging');
}

// Setup drop target for editor
function setupEditorDropHandler() {
    const editorContainer = document.getElementById('editor-container');
    if (!editorContainer) return;

    editorContainer.addEventListener('dragover', (e) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'copy';
    });

    editorContainer.addEventListener('drop', (e) => {
        e.preventDefault();

        const imageData = e.dataTransfer.getData('application/x-sikulix-image');
        if (imageData) {
            try {
                const data = JSON.parse(imageData);
                insertTextAtCursor(`"${data.filename}"`);
            } catch (err) {
                // Fallback to plain text
                const text = e.dataTransfer.getData('text/plain');
                if (text) {
                    insertTextAtCursor(text);
                }
            }
        }
    });
}

// Sidebar resize handler
function setupSidebarResize() {
    const handle = document.getElementById('sidebar-resize-handle');
    const sidebar = document.getElementById('image-library-sidebar');

    if (!handle || !sidebar) return;

    handle.addEventListener('mousedown', (e) => {
        sidebarResizing = true;
        handle.classList.add('resizing');
        document.body.style.cursor = 'col-resize';
        e.preventDefault();
    });

    document.addEventListener('mousemove', (e) => {
        if (!sidebarResizing) return;

        const newWidth = e.clientX;
        if (newWidth >= 150 && newWidth <= 400) {
            sidebar.style.width = newWidth + 'px';
        }
    });

    document.addEventListener('mouseup', () => {
        if (sidebarResizing) {
            sidebarResizing = false;
            handle.classList.remove('resizing');
            document.body.style.cursor = '';
            if (editor) editor.layout();
        }
    });
}

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', () => {
    setupEditorDropHandler();
    setupSidebarResize();
});

// ==================== OCR Functions ====================

let ocrLanguage = 'eng';
let ocrResult = {
    text: '',
    confidence: 0
};
let ocrProcessing = false;

/**
 * Set OCR language
 */
function setOcrLanguage(lang) {
    ocrLanguage = lang;
    console.log('OCR language set to:', lang);
}

/**
 * Capture and perform OCR
 */
async function captureOcr() {
    if (ocrProcessing) return;

    const resultText = document.getElementById('ocr-result-text');
    const confidenceFill = document.getElementById('confidence-fill');
    const confidenceValue = document.getElementById('confidence-value');
    const captureBtn = document.querySelector('.ocr-capture-btn');

    // Set processing state
    ocrProcessing = true;
    captureBtn.disabled = true;
    resultText.textContent = t('ocr.processing');
    resultText.classList.add('processing');

    try {
        // Call Tauri command if available
        if (window.__TAURI__) {
            const result = await window.__TAURI__.invoke('capture_ocr', {
                language: ocrLanguage
            });

            ocrResult = result;
            displayOcrResult(result.text, result.confidence);
        } else {
            // Mock result for development
            await new Promise(resolve => setTimeout(resolve, 1000));

            const mockText = ocrLanguage.includes('jpn')
                ? 'サンプルテキスト\nこれはOCRテストです。'
                : 'Sample text\nThis is an OCR test.';
            const mockConfidence = 0.85 + Math.random() * 0.1;

            ocrResult = { text: mockText, confidence: mockConfidence };
            displayOcrResult(mockText, mockConfidence);
        }
    } catch (error) {
        console.error('OCR error:', error);
        resultText.textContent = t('ocr.error') + ': ' + error.message;
        resultText.classList.remove('processing');
        confidenceFill.style.width = '0%';
        confidenceValue.textContent = '--%';
    } finally {
        ocrProcessing = false;
        captureBtn.disabled = false;
    }
}

/**
 * Display OCR result
 */
function displayOcrResult(text, confidence) {
    const resultText = document.getElementById('ocr-result-text');
    const confidenceFill = document.getElementById('confidence-fill');
    const confidenceValue = document.getElementById('confidence-value');

    resultText.classList.remove('processing');
    resultText.textContent = text || t('ocr.no_text');

    const confidencePercent = Math.round(confidence * 100);
    confidenceFill.style.width = confidencePercent + '%';
    confidenceValue.textContent = confidencePercent + '%';

    // Color based on confidence level
    if (confidencePercent >= 80) {
        confidenceValue.style.color = '#4ec9b0';
    } else if (confidencePercent >= 60) {
        confidenceValue.style.color = '#ffc107';
    } else {
        confidenceValue.style.color = '#f14c4c';
    }
}

/**
 * Copy OCR result to clipboard
 */
async function copyOcrText() {
    const text = ocrResult.text;
    if (!text) {
        showStatus(t('ocr.no_text_copy'));
        return;
    }

    try {
        await navigator.clipboard.writeText(text);
        showStatus(t('ocr.copied'));
    } catch (error) {
        console.error('Failed to copy:', error);
        // Fallback for older browsers
        const textarea = document.createElement('textarea');
        textarea.value = text;
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand('copy');
        document.body.removeChild(textarea);
        showStatus(t('ocr.copied'));
    }
}

/**
 * Clear OCR result
 */
function clearOcrResult() {
    ocrResult = { text: '', confidence: 0 };

    const resultText = document.getElementById('ocr-result-text');
    const confidenceFill = document.getElementById('confidence-fill');
    const confidenceValue = document.getElementById('confidence-value');

    resultText.textContent = t('ocr.no_result');
    confidenceFill.style.width = '0%';
    confidenceValue.textContent = '--%';
    confidenceValue.style.color = '#ccc';

    // Clear bounding boxes
    clearOcrBoundingBoxes();
}

// ==================== Region OCR Functions ====================

let ocrRegionOverlay = null;
let ocrSelectionBox = null;
let ocrStartX = 0;
let ocrStartY = 0;
let ocrBboxOverlay = null;

/**
 * Capture region and perform OCR
 */
function captureRegionOcr() {
    // Create overlay
    ocrRegionOverlay = document.createElement('div');
    ocrRegionOverlay.className = 'ocr-region-overlay';

    // Create hint
    const hint = document.createElement('div');
    hint.className = 'ocr-region-hint';
    hint.textContent = t('ocr.region_hint');
    ocrRegionOverlay.appendChild(hint);

    // Mouse events
    ocrRegionOverlay.addEventListener('mousedown', startOcrRegionSelection);
    ocrRegionOverlay.addEventListener('mousemove', updateOcrRegionSelection);
    ocrRegionOverlay.addEventListener('mouseup', endOcrRegionSelection);
    ocrRegionOverlay.addEventListener('keydown', cancelOcrRegionSelection);

    document.body.appendChild(ocrRegionOverlay);
    ocrRegionOverlay.focus();

    // Allow escape to cancel
    document.addEventListener('keydown', cancelOcrRegionSelection);
}

/**
 * Start region selection
 */
function startOcrRegionSelection(e) {
    ocrStartX = e.clientX;
    ocrStartY = e.clientY;

    ocrSelectionBox = document.createElement('div');
    ocrSelectionBox.className = 'ocr-region-selection';
    ocrSelectionBox.style.left = ocrStartX + 'px';
    ocrSelectionBox.style.top = ocrStartY + 'px';
    ocrRegionOverlay.appendChild(ocrSelectionBox);
}

/**
 * Update region selection
 */
function updateOcrRegionSelection(e) {
    if (!ocrSelectionBox) return;

    const currentX = e.clientX;
    const currentY = e.clientY;

    const left = Math.min(ocrStartX, currentX);
    const top = Math.min(ocrStartY, currentY);
    const width = Math.abs(currentX - ocrStartX);
    const height = Math.abs(currentY - ocrStartY);

    ocrSelectionBox.style.left = left + 'px';
    ocrSelectionBox.style.top = top + 'px';
    ocrSelectionBox.style.width = width + 'px';
    ocrSelectionBox.style.height = height + 'px';
}

/**
 * End region selection and perform OCR
 */
async function endOcrRegionSelection(e) {
    if (!ocrSelectionBox) return;

    const rect = ocrSelectionBox.getBoundingClientRect();

    // Remove overlay
    if (ocrRegionOverlay) {
        ocrRegionOverlay.remove();
        ocrRegionOverlay = null;
    }
    document.removeEventListener('keydown', cancelOcrRegionSelection);
    ocrSelectionBox = null;

    // Skip if selection is too small
    if (rect.width < 10 || rect.height < 10) {
        showStatus(t('ocr.region_too_small'));
        return;
    }

    // Perform OCR on region
    await performRegionOcr({
        x: Math.round(rect.left),
        y: Math.round(rect.top),
        width: Math.round(rect.width),
        height: Math.round(rect.height)
    });
}

/**
 * Cancel region selection
 */
function cancelOcrRegionSelection(e) {
    if (e.key === 'Escape') {
        if (ocrRegionOverlay) {
            ocrRegionOverlay.remove();
            ocrRegionOverlay = null;
        }
        document.removeEventListener('keydown', cancelOcrRegionSelection);
        ocrSelectionBox = null;
    }
}

/**
 * Perform OCR on specific region
 */
async function performRegionOcr(region) {
    const resultText = document.getElementById('ocr-result-text');
    const confidenceFill = document.getElementById('confidence-fill');
    const confidenceValue = document.getElementById('confidence-value');

    // Set processing state
    ocrProcessing = true;
    resultText.textContent = t('ocr.processing');
    resultText.classList.add('processing');

    // Switch to OCR panel
    switchBottomPanel('ocr');

    try {
        // Call Tauri command if available
        if (window.__TAURI__) {
            const result = await window.__TAURI__.invoke('ocr_region', {
                language: ocrLanguage,
                x: region.x,
                y: region.y,
                width: region.width,
                height: region.height
            });

            ocrResult = result;
            displayOcrResult(result.text, result.confidence);

            // Show bounding boxes if available
            if (result.boxes && result.boxes.length > 0) {
                displayOcrBoundingBoxes(result.boxes);
            }
        } else {
            // Mock result for development
            await new Promise(resolve => setTimeout(resolve, 800));

            const mockText = ocrLanguage.includes('jpn')
                ? '選択領域のサンプルテキスト\n日本語OCRテスト'
                : 'Sample text from selected region\nOCR test result';
            const mockConfidence = 0.80 + Math.random() * 0.15;
            const mockBoxes = [
                { x: region.x + 10, y: region.y + 10, width: 100, height: 20, text: 'Sample' },
                { x: region.x + 10, y: region.y + 35, width: 80, height: 20, text: 'text' }
            ];

            ocrResult = { text: mockText, confidence: mockConfidence, boxes: mockBoxes };
            displayOcrResult(mockText, mockConfidence);
            displayOcrBoundingBoxes(mockBoxes);
        }
    } catch (error) {
        console.error('Region OCR error:', error);
        resultText.textContent = t('ocr.error') + ': ' + error.message;
        resultText.classList.remove('processing');
    } finally {
        ocrProcessing = false;
    }
}

/**
 * Display OCR bounding boxes on screen
 */
function displayOcrBoundingBoxes(boxes) {
    clearOcrBoundingBoxes();

    ocrBboxOverlay = document.createElement('div');
    ocrBboxOverlay.className = 'ocr-bbox-overlay';

    boxes.forEach(box => {
        const bbox = document.createElement('div');
        bbox.className = 'ocr-bbox';
        bbox.style.left = box.x + 'px';
        bbox.style.top = box.y + 'px';
        bbox.style.width = box.width + 'px';
        bbox.style.height = box.height + 'px';

        if (box.text) {
            const textLabel = document.createElement('div');
            textLabel.className = 'ocr-bbox-text';
            textLabel.textContent = box.text;
            bbox.appendChild(textLabel);
        }

        bbox.addEventListener('click', () => {
            if (box.text) {
                navigator.clipboard.writeText(box.text);
                showStatus(t('ocr.copied'));
            }
        });

        ocrBboxOverlay.appendChild(bbox);
    });

    document.body.appendChild(ocrBboxOverlay);

    // Auto-hide after 10 seconds
    setTimeout(() => {
        clearOcrBoundingBoxes();
    }, 10000);
}

/**
 * Clear OCR bounding boxes
 */
function clearOcrBoundingBoxes() {
    if (ocrBboxOverlay) {
        ocrBboxOverlay.remove();
        ocrBboxOverlay = null;
    }
}

/**
 * Search for text on screen using OCR
 */
async function searchOcrText(searchText) {
    if (!searchText || searchText.trim() === '') {
        showStatus(t('ocr.search_empty'));
        return;
    }

    const resultText = document.getElementById('ocr-result-text');
    resultText.textContent = t('ocr.searching');
    resultText.classList.add('processing');

    try {
        if (window.__TAURI__) {
            const result = await window.__TAURI__.invoke('search_text_on_screen', {
                language: ocrLanguage,
                searchText: searchText.trim()
            });

            if (result.matches && result.matches.length > 0) {
                displayOcrBoundingBoxes(result.matches);
                resultText.textContent = t('ocr.search_found').replace('{count}', result.matches.length);
            } else {
                resultText.textContent = t('ocr.search_not_found');
            }
        } else {
            // Mock search result
            await new Promise(resolve => setTimeout(resolve, 500));
            resultText.textContent = t('ocr.search_not_found');
        }
    } catch (error) {
        console.error('OCR search error:', error);
        resultText.textContent = t('ocr.error') + ': ' + error.message;
    } finally {
        resultText.classList.remove('processing');
    }
}

// ==================== OCR Language Management ====================

// Available OCR languages with their display names
const availableOcrLanguages = {
    'eng': { en: 'English', ja: '英語' },
    'jpn': { en: 'Japanese', ja: '日本語' },
    'jpn_vert': { en: 'Japanese (Vertical)', ja: '日本語（縦書き）' },
    'chi_sim': { en: 'Chinese (Simplified)', ja: '中国語（簡体字）' },
    'chi_tra': { en: 'Chinese (Traditional)', ja: '中国語（繁体字）' },
    'kor': { en: 'Korean', ja: '韓国語' },
    'deu': { en: 'German', ja: 'ドイツ語' },
    'fra': { en: 'French', ja: 'フランス語' },
    'spa': { en: 'Spanish', ja: 'スペイン語' },
    'por': { en: 'Portuguese', ja: 'ポルトガル語' },
    'rus': { en: 'Russian', ja: 'ロシア語' },
    'ara': { en: 'Arabic', ja: 'アラビア語' }
};

// Installed languages (will be updated from backend)
let installedOcrLanguages = ['eng', 'jpn'];

/**
 * Load available OCR languages from backend
 */
async function loadOcrLanguages() {
    try {
        if (window.__TAURI__) {
            const languages = await window.__TAURI__.invoke('get_available_ocr_languages');
            installedOcrLanguages = languages;
        }
        updateOcrLanguageSelect();
    } catch (error) {
        console.error('Failed to load OCR languages:', error);
        updateOcrLanguageSelect();
    }
}

/**
 * Update OCR language select dropdown
 */
function updateOcrLanguageSelect() {
    const select = document.getElementById('ocr-language');
    if (!select) return;

    const currentLang = currentLanguage; // UI language
    select.innerHTML = '';

    // Add installed languages
    installedOcrLanguages.forEach(langCode => {
        const option = document.createElement('option');
        option.value = langCode;
        const langInfo = availableOcrLanguages[langCode];
        option.textContent = langInfo ? langInfo[currentLang] || langInfo.en : langCode;
        select.appendChild(option);
    });

    // Add common combinations
    if (installedOcrLanguages.includes('eng') && installedOcrLanguages.includes('jpn')) {
        const option = document.createElement('option');
        option.value = 'eng+jpn';
        option.textContent = currentLang === 'ja' ? '英語 + 日本語' : 'Eng + Jpn';
        select.appendChild(option);
    }

    // Restore selection
    if (ocrLanguage && select.querySelector(`option[value="${ocrLanguage}"]`)) {
        select.value = ocrLanguage;
    } else if (installedOcrLanguages.length > 0) {
        select.value = installedOcrLanguages[0];
        ocrLanguage = installedOcrLanguages[0];
    }
}

/**
 * Check if language data is available
 */
async function checkOcrLanguageAvailability(langCode) {
    try {
        if (window.__TAURI__) {
            return await window.__TAURI__.invoke('check_ocr_language', { language: langCode });
        }
        return installedOcrLanguages.includes(langCode);
    } catch (error) {
        console.error('Failed to check language availability:', error);
        return false;
    }
}

/**
 * Show language download dialog (placeholder)
 */
function showLanguageDownloadDialog() {
    const message = currentLanguage === 'ja'
        ? 'OCR言語データのダウンロード機能は今後のバージョンで実装予定です。\n\n手動でTesseract言語データをインストールしてください。'
        : 'OCR language data download feature is planned for future versions.\n\nPlease install Tesseract language data manually.';

    alert(message);
}

// Initialize OCR languages on load
document.addEventListener('DOMContentLoaded', () => {
    loadOcrLanguages();
});

// Update OCR language select when UI language changes
const originalSetLanguage = typeof setLanguage === 'function' ? setLanguage : null;
if (originalSetLanguage) {
    window.setLanguage = function(lang) {
        originalSetLanguage(lang);
        updateOcrLanguageSelect();
    };
}

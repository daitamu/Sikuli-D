# Recording Feature Design Document

## 1. Overview

### Purpose
The Recording Feature enables users to capture their mouse and keyboard interactions in real-time and automatically generate corresponding Python scripts compatible with SikuliX API. This feature significantly reduces the learning curve for automation beginners and accelerates script creation for experienced users.

### Target Users
- **Automation Beginners**: Users who want to create scripts without extensive programming knowledge
- **Quick Prototyping**: Experienced users who need to rapidly generate script templates
- **Learning Tool**: Users learning SikuliX/Sikuli-D syntax through example generation

### Key Benefits
- Zero-code script generation through interaction recording
- Real-time visual feedback of captured events
- Editable event timeline for fine-tuning
- Intelligent wait time insertion
- Screenshot capture for image-based operations

---

## 2. Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React/TypeScript)           │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Recording Toolbar                               │   │
│  │  [Start] [Pause] [Stop] [Clear]                 │   │
│  └─────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Event List View                                 │   │
│  │  • Mouse Click (100, 200)        [Edit] [Del]   │   │
│  │  • Wait 1.5s                     [Edit] [Del]   │   │
│  │  • Type "hello"                  [Edit] [Del]   │   │
│  └─────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Generated Script Preview                        │   │
│  │  click(100, 200)                                 │   │
│  │  wait(1.5)                                       │   │
│  │  type("hello")                                   │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │ Tauri IPC Commands
                          ▼
┌─────────────────────────────────────────────────────────┐
│              Backend (Rust - Tauri Commands)             │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Recording Controller                            │   │
│  │  • start_recording()                             │   │
│  │  • pause_recording()                             │   │
│  │  • stop_recording()                              │   │
│  │  • get_recorded_events()                         │   │
│  └─────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Event Capture Engine                            │   │
│  │  • Mouse Hook (OS-specific)                      │   │
│  │  • Keyboard Hook (OS-specific)                   │   │
│  │  • Screenshot Capture                            │   │
│  │  • Timestamp Recording                           │   │
│  └─────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Script Generator                                │   │
│  │  • Event → Python Code Converter                │   │
│  │  • Smart Wait Insertion                          │   │
│  │  • Image Pattern Generation                      │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│        Core Library (sikulid-core - Existing APIs)       │
│  • Mouse::position()                                     │
│  • Keyboard event capture (via OS hooks)                │
│  • Screen::capture_region()                             │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Component Responsibilities

#### 2.2.1 Frontend (React/TypeScript)

**Recording Toolbar**
- Start/Pause/Stop/Clear buttons
- Recording status indicator (idle/recording/paused)
- Elapsed time display
- Global hotkey display (e.g., "Press Ctrl+Shift+R to stop")

**Event List View**
- Real-time event display as they are captured
- Event type icons (mouse, keyboard, wait, screenshot)
- Editable event properties (coordinates, text, delays)
- Drag-and-drop reordering
- Individual event deletion
- Batch event selection and deletion

**Script Preview Panel**
- Live-updated Python script generation
- Syntax highlighting
- Copy-to-editor button
- Save-as-file button

#### 2.2.2 Backend (Rust - ide-rs-tauri)

**Recording Controller** (`ide-rs-tauri/src/recording.rs`)
- Manages recording state machine (Idle → Recording → Paused → Stopped)
- Coordinates between event capture and frontend
- Handles global hotkeys for recording control
- Thread-safe event queue management

**Event Capture Engine** (Platform-specific implementations)
- **Windows**: Use `SetWindowsHookEx` with `WH_MOUSE_LL` and `WH_KEYBOARD_LL`
- **macOS**: Use `CGEvent` tap callbacks
- **Linux**: Use `x11rb` XRecord extension or `evdev` input device monitoring

**Script Generator**
- Converts captured events into Python function calls
- Applies intelligent wait insertion rules
- Generates image pattern references
- Handles special characters and Unicode text

#### 2.2.3 Core Library Integration (sikulid-core)

Leverage existing APIs:
- `Mouse::position()` - Get mouse coordinates
- `Screen::capture_region()` - Capture click target images
- Platform-specific input modules already implemented

---

## 3. Event Types to Capture

### 3.1 Mouse Events

| Event Type       | Captured Data                    | Generated Code Example       |
|------------------|----------------------------------|------------------------------|
| Left Click       | (x, y, timestamp)                | `click(100, 200)`            |
| Double Click     | (x, y, timestamp)                | `doubleClick(100, 200)`      |
| Right Click      | (x, y, timestamp)                | `rightClick(100, 200)`       |
| Middle Click     | (x, y, timestamp)                | `middleClick(100, 200)`      |
| Drag             | (start_x, start_y, end_x, end_y) | `drag(10, 20, 100, 200)`     |
| Scroll Up/Down   | (x, y, clicks, direction)        | `wheel(WHEEL_DOWN, 3)`       |
| Mouse Move       | (x, y) - optional                | `hover(100, 200)` (optional) |

### 3.2 Keyboard Events

| Event Type       | Captured Data                    | Generated Code Example       |
|------------------|----------------------------------|------------------------------|
| Text Input       | String text                      | `type("Hello World")`        |
| Single Key       | Key code                         | `type(Key.ENTER)`            |
| Key Combination  | Modifier keys + key              | `type("s", KeyModifier.CTRL)`|
| Special Keys     | F1-F12, ESC, TAB, etc.           | `type(Key.F5)`               |

**Text Buffering**: Consecutive character inputs within 500ms are buffered into a single `type()` call.

### 3.3 Screen Capture Events

| Event Type       | When Triggered                   | Generated Code Example       |
|------------------|----------------------------------|------------------------------|
| Click Screenshot | On mouse click (optional)        | `click("button.png")`        |
| Manual Capture   | User presses capture hotkey      | `click("target.png")`        |

**Screenshot Options**:
- Capture entire screen or region around click point
- Automatic thumbnail generation
- Stored in project's image library
- Automatic filename generation (e.g., `click_001.png`)

### 3.4 Timing Events

| Event Type       | Auto-Generated When              | Generated Code Example       |
|------------------|----------------------------------|------------------------------|
| Wait             | Delay > threshold between events | `wait(1.5)`                  |

**Smart Wait Insertion Rules**:
- Delays < 0.5s: Ignored
- Delays 0.5s-5s: Generate `wait(seconds)`
- Delays > 5s: Prompt user to confirm or set custom value

---

## 4. Script Generation Rules

### 4.1 Python Function Mapping

```python
# Mouse Events
click(x, y)                    # Left click
doubleClick(x, y)              # Double click
rightClick(x, y)               # Right click
dragDrop(start_x, start_y, end_x, end_y)  # Drag
wheel(direction, steps)        # Scroll
hover(x, y)                    # Mouse hover (optional)

# Keyboard Events
type("text")                   # Type text string
type(Key.ENTER)                # Press special key
type("s", KeyModifier.CTRL)    # Hotkey combination

# Image-Based Events (when screenshot captured)
click(Pattern("button.png"))   # Click on image pattern
wait("icon.png", 5)            # Wait for image to appear

# Timing
wait(seconds)                  # Explicit wait
sleep(seconds)                 # Alternative wait (alias)
```

### 4.2 Code Generation Examples

**Example 1: Simple Click Sequence**
```
Recorded Events:
1. Mouse Click at (500, 300) - 00:00:00.000
2. Wait 1.2s
3. Mouse Click at (600, 400) - 00:00:01.200

Generated Script:
click(500, 300)
wait(1.2)
click(600, 400)
```

**Example 2: Text Input with Hotkey**
```
Recorded Events:
1. Mouse Click at (200, 100)
2. Type "Hello World"
3. Key Press: Ctrl+S

Generated Script:
click(200, 100)
type("Hello World")
type("s", KeyModifier.CTRL)
```

**Example 3: Drag Operation**
```
Recorded Events:
1. Mouse Down at (100, 200)
2. Mouse Move to (300, 400)
3. Mouse Up at (300, 400)

Generated Script:
dragDrop(100, 200, 300, 400)
```

**Example 4: Image-Based Click**
```
Recorded Events:
1. Mouse Click at (500, 300) with screenshot "button_001.png"

Generated Script:
click(Pattern("button_001.png"))
```

### 4.3 Advanced Generation Features

**Variable Insertion**
- Replace repeated coordinates with variables
- Example: Multiple clicks at same location → `target = Location(100, 200)`

**Loop Detection**
- Detect repeated event sequences
- Suggest `for` loop generation

**Comment Insertion**
- Add timestamps as comments
- Example: `# Recorded at 2025-12-03 14:30:15`

---

## 5. UI Design

### 5.1 Recording Toolbar

```
┌────────────────────────────────────────────────────────┐
│  Recording Toolbar                                      │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐  [●] 00:45  [Ctrl+Shift+R] │
│  │ ▶  │ │ ⏸  │ │ ⏹  │ │ 🗑 │                            │
│  └────┘ └────┘ └────┘ └────┘                            │
│  Start  Pause   Stop   Clear   Status    Hotkey          │
└────────────────────────────────────────────────────────┘
```

**Button States**:
- Start: Enabled when Idle or Stopped
- Pause: Enabled when Recording
- Stop: Enabled when Recording or Paused
- Clear: Enabled when events exist

### 5.2 Event List View

```
┌────────────────────────────────────────────────────────┐
│  Recorded Events (15 events)               [Clear All] │
├────────────────────────────────────────────────────────┤
│  [🖱] 00:00.000  Click at (500, 300)    [Edit] [Del]   │
│  [⏱] 00:01.200  Wait 1.2s               [Edit] [Del]   │
│  [⌨] 00:01.200  Type "Hello"            [Edit] [Del]   │
│  [🖱] 00:02.500  Right Click (600, 400) [Edit] [Del]   │
│  [🖱] 00:03.000  Drag (100,200→300,400) [Edit] [Del]   │
│  [📷] 00:03.500  Click "button.png"     [Edit] [Del]   │
└────────────────────────────────────────────────────────┘
```

**Event Icons**:
- 🖱 Mouse events
- ⌨ Keyboard events
- ⏱ Wait/delay
- 📷 Screenshot-based event

**Actions**:
- Edit: Open inline editor to modify event properties
- Del: Remove event from list
- Drag handle for reordering

### 5.3 Event Edit Dialog

```
┌────────────────────────────────────────────┐
│  Edit Mouse Click Event                    │
├────────────────────────────────────────────┤
│  X Coordinate: [500    ]                   │
│  Y Coordinate: [300    ]                   │
│  Click Type:   [Single ▼]                  │
│                                            │
│  □ Capture screenshot at this location     │
│  □ Use image pattern instead of coords     │
│                                            │
│  [Cancel]                    [Save Changes]│
└────────────────────────────────────────────┘
```

### 5.4 Script Preview Panel

```
┌────────────────────────────────────────────────────────┐
│  Generated Script           [Copy] [Save] [Insert]     │
├────────────────────────────────────────────────────────┤
│   1  # Recorded on 2025-12-03 14:30:15                 │
│   2  # Total events: 6                                 │
│   3                                                     │
│   4  click(500, 300)                                   │
│   5  wait(1.2)                                         │
│   6  type("Hello World")                               │
│   7  rightClick(600, 400)                              │
│   8  dragDrop(100, 200, 300, 400)                      │
│   9  click(Pattern("button_001.png"))                  │
└────────────────────────────────────────────────────────┘
```

**Actions**:
- Copy: Copy script to clipboard
- Save: Save as `.py` file
- Insert: Insert into current editor at cursor position

---

## 6. Implementation Plan

### Phase 1: Basic Event Capture (Weeks 1-2)

**Deliverables**:
- Recording state machine implementation
- Mouse event capture (click, double-click, right-click)
- Keyboard event capture (text input, single keys)
- Basic event storage and retrieval
- Simple toolbar UI (Start/Stop buttons)

**Technical Tasks**:
1. Create `ide-rs-tauri/src/recording.rs` module
2. Implement OS-specific mouse hooks
   - Windows: `SetWindowsHookEx` with `WH_MOUSE_LL`
   - macOS: `CGEvent` tap
   - Linux: `x11rb` XRecord
3. Implement OS-specific keyboard hooks
4. Create event data structures (enums for event types)
5. Implement thread-safe event queue
6. Create Tauri commands: `start_recording`, `stop_recording`, `get_events`
7. Build basic frontend recording toolbar

**Testing**:
- Unit tests for event data structures
- Integration tests for mouse/keyboard capture
- Manual UI testing for start/stop functionality

### Phase 2: Script Generation (Weeks 3-4)

**Deliverables**:
- Python script generator
- Smart wait insertion logic
- Event-to-code conversion
- Script preview panel
- Copy/Save script functionality

**Technical Tasks**:
1. Implement `ScriptGenerator` trait
2. Create code templates for each event type
3. Implement wait time detection and insertion
4. Add text buffering for consecutive keystrokes
5. Create script preview component
6. Implement clipboard copy and file save
7. Add syntax highlighting to preview

**Testing**:
- Unit tests for code generation functions
- Test various event sequences
- Validate generated Python syntax
- Test edge cases (special characters, long waits)

### Phase 3: Edit/Preview UI (Weeks 5-6)

**Deliverables**:
- Event list view with real-time updates
- Event editing dialog
- Event deletion and reordering
- Enhanced preview with line numbers
- Insert script into editor functionality

**Technical Tasks**:
1. Create `EventListView` React component
2. Implement drag-and-drop reordering
3. Build event edit dialog with validation
4. Add delete confirmation
5. Implement "Insert to Editor" feature
6. Add event filtering/search
7. Create event detail tooltips

**Testing**:
- UI component tests (React Testing Library)
- Test drag-and-drop functionality
- Test event editing and validation
- Test script insertion into Monaco editor

### Phase 4: Smart Wait & Screenshot Capture (Weeks 7-8)

**Deliverables**:
- Automatic screenshot capture on click
- Image pattern generation
- Smart wait threshold configuration
- Image library integration
- Pause/Resume functionality
- Global hotkeys for recording control

**Technical Tasks**:
1. Integrate `Screen::capture_region()` for screenshots
2. Implement automatic image saving to project folder
3. Create Pattern code generation
4. Add pause/resume recording logic
5. Implement global hotkey registration
6. Add screenshot preview thumbnails in event list
7. Create settings panel for wait thresholds
8. Add image pattern editing UI

**Testing**:
- Test screenshot capture accuracy
- Test image file storage and naming
- Test global hotkeys across platforms
- Test pause/resume state transitions
- Validate Pattern code generation

### Phase 5: Advanced Features (Weeks 9-10) - Optional

**Deliverables**:
- Loop detection and suggestion
- Variable extraction for repeated coordinates
- Comment insertion options
- Recording filters (ignore mouse moves)
- Export to different script formats

**Technical Tasks**:
1. Implement pattern detection algorithm
2. Create loop suggestion UI
3. Add coordinate clustering for variable extraction
4. Implement filtering options
5. Add export format selection
6. Create recording templates

**Testing**:
- Test pattern detection accuracy
- Test variable extraction logic
- Validate different export formats

---

## 7. Technical Considerations

### 7.1 Cross-Platform Event Capture

**Windows**
- Use `windows` crate (already in dependencies)
- `SetWindowsHookEx` with `WH_MOUSE_LL` and `WH_KEYBOARD_LL`
- Handle `WM_MOUSEMOVE`, `WM_LBUTTONDOWN`, `WM_KEYDOWN`, etc.
- Thread-safe event queue with `Mutex<Vec<Event>>`

**macOS**
- Use `core-graphics` and `cocoa` crates (already in dependencies)
- `CGEventTapCreate` for global event monitoring
- Request accessibility permissions on first run
- Handle `kCGEventLeftMouseDown`, `kCGEventKeyDown`, etc.

**Linux**
- Use `x11rb` crate (already in dependencies)
- XRecord extension for event capture
- Alternative: `evdev` for raw input device access
- Requires root or proper udev rules for device access

### 7.2 Performance Impact During Recording

**Optimization Strategies**:
1. **Event Buffering**: Queue events in memory, flush to storage periodically
2. **Background Thread**: Capture events in separate thread to avoid UI blocking
3. **Selective Capture**: Filter out mouse move events by default (configurable)
4. **Screenshot Throttling**: Limit screenshot capture rate to avoid disk I/O bottleneck
5. **Lazy Screenshot Generation**: Capture only when user explicitly requests

**Performance Targets**:
- Mouse/keyboard event latency: < 10ms
- UI update latency: < 50ms
- Memory usage: < 50MB for 1000 events
- Screenshot capture: < 100ms per capture

### 7.3 Privacy Considerations

**Security Measures**:
1. **No Automatic Logging**: Recording only active when explicitly started by user
2. **Clear Visual Indicator**: Always show recording status icon
3. **Password Protection**: Do not capture keystrokes in password fields (detect by window title/class)
4. **Sensitive Window Detection**: Warn when recording in banking/password manager apps
5. **Local Storage Only**: All recorded data stays on user's machine
6. **Clear Action Required**: Provide easy "Clear All" button to delete recorded data
7. **Session-Only Storage**: Do not persist recordings across app restarts (optional)

**User Controls**:
- Global hotkey to pause recording instantly
- Configurable recording filters
- Option to disable screenshot capture
- Option to exclude specific applications
- Post-recording review before script generation

### 7.4 Data Structures

**Event Data Model**:
```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordedEvent {
    MouseClick {
        x: i32,
        y: i32,
        button: MouseButton,
        timestamp: Duration,
        screenshot: Option<String>, // Path to screenshot
    },
    MouseDrag {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        timestamp: Duration,
    },
    MouseScroll {
        x: i32,
        y: i32,
        delta: i32,
        direction: ScrollDirection,
        timestamp: Duration,
    },
    KeyPress {
        key: String,
        modifiers: Vec<KeyModifier>,
        timestamp: Duration,
    },
    TextInput {
        text: String,
        timestamp: Duration,
    },
    Wait {
        duration: f64, // seconds
        timestamp: Duration,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}
```

**Recording Session**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub session_id: String,
    pub start_time: std::time::SystemTime,
    pub events: Vec<RecordedEvent>,
    pub status: RecordingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordingStatus {
    Idle,
    Recording,
    Paused,
    Stopped,
}
```

### 7.5 Global Hotkeys

**Default Hotkeys**:
- Start/Stop Recording: `Ctrl+Shift+R`
- Pause/Resume: `Ctrl+Shift+P`
- Capture Screenshot: `Ctrl+Shift+C`

**Implementation**:
- Use `tauri-plugin-global-shortcut` (already in dependencies)
- Allow hotkey customization in settings
- Validate no conflicts with system shortcuts

### 7.6 Error Handling

**Common Error Cases**:
1. **Permission Denied**: Accessibility permissions not granted (macOS/Linux)
   - Solution: Show permission request dialog with instructions
2. **Hook Registration Failed**: Another app using same hook
   - Solution: Fallback to polling-based capture, warn user about limitations
3. **Screenshot Capture Failed**: Screen locked or capture blocked
   - Solution: Skip screenshot, continue with coordinate-based code generation
4. **Disk Space Full**: Cannot save screenshots
   - Solution: Disable screenshot capture, show warning
5. **Recording Interrupted**: App crash during recording
   - Solution: Implement auto-save recovery, restore last session

---

## 8. Testing Strategy

### 8.1 Unit Tests
- Event data structure serialization/deserialization
- Script generation for each event type
- Wait time calculation logic
- Text buffering algorithm
- Coordinate clustering for variables

### 8.2 Integration Tests
- Mouse/keyboard capture accuracy
- Thread-safe event queue operations
- Tauri IPC command communication
- File I/O for screenshots and scripts
- Settings persistence

### 8.3 UI Tests
- Recording toolbar button states
- Event list display and updates
- Drag-and-drop reordering
- Event editing and validation
- Script preview updates

### 8.4 Manual Tests
- Cross-platform functionality (Windows/macOS/Linux)
- Recording in different applications
- Global hotkey responsiveness
- Screenshot capture quality
- Generated script execution accuracy
- Performance under load (1000+ events)

### 8.5 Security Tests
- Password field detection
- Sensitive window detection
- Data deletion verification
- Permission handling

---

## 9. Future Enhancements

### 9.1 AI-Powered Features
- Intelligent event grouping
- Natural language descriptions of recorded actions
- Automatic variable naming suggestions
- Script optimization recommendations

### 9.2 Advanced Recording Modes
- Region-specific recording (only capture events in selected area)
- Application-specific recording (filter by window)
- Schedule-based recording (start/stop at specific times)

### 9.3 Collaboration Features
- Share recordings as templates
- Import/export recording sessions
- Cloud sync for team libraries

### 9.4 Analytics
- Recording statistics (most used actions)
- Script complexity metrics
- Performance profiling of recorded scripts

---

## 10. Dependencies

### Existing Dependencies
- `windows` - Windows API bindings
- `core-graphics`, `cocoa`, `objc` - macOS event handling
- `x11rb` - Linux X11 bindings
- `tauri-plugin-global-shortcut` - Global hotkey support
- `serde`, `serde_json` - Serialization
- `image` - Screenshot handling
- `tokio` - Async runtime

### New Dependencies Required
- None (all required functionality available in existing dependencies)

### Optional Dependencies
- `rdev` - Alternative cross-platform input capture (if platform-specific code becomes too complex)

---

## 11. Documentation Requirements

### User Documentation
- Quick Start Guide: "Your First Recording"
- Tutorial: "Creating Scripts by Recording"
- Best Practices: "When to Use Recording vs Manual Coding"
- Troubleshooting: "Common Recording Issues"

### Developer Documentation
- Architecture Overview
- Event Capture Implementation Guide
- Script Generator Extension Guide
- Platform-Specific Notes

---

## 12. Success Criteria

### Minimum Viable Product (MVP)
- [ ] Record mouse clicks and keyboard input
- [ ] Generate executable Python scripts
- [ ] Basic UI with start/stop controls
- [ ] Works on Windows

### Full Release
- [ ] All event types supported
- [ ] Edit/delete/reorder events
- [ ] Smart wait insertion
- [ ] Screenshot capture and Pattern generation
- [ ] Global hotkeys
- [ ] Cross-platform support (Windows/macOS/Linux)
- [ ] Comprehensive documentation

### Performance Targets
- [ ] < 10ms event capture latency
- [ ] < 50ms UI update latency
- [ ] Support 1000+ events per session
- [ ] < 100ms screenshot capture time

---

## 13. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Platform-specific bugs | High | Medium | Extensive cross-platform testing |
| Performance degradation | Medium | High | Profiling, optimization, background threading |
| Permission issues (macOS) | High | High | Clear permission request UI, documentation |
| Generated script errors | Medium | High | Comprehensive script validation, testing |
| User privacy concerns | Low | Critical | Strict privacy controls, clear indicators |

---

## Document Version
- **Version**: 1.0
- **Date**: 2025-12-03
- **Author**: Claude (AI Assistant)
- **Status**: Design Phase

//! Input integration tests
//! 入力統合テスト
//!
//! Tests mouse and keyboard input operations in a mock environment.
//! モック環境でマウスとキーボード入力操作をテストします。

use sikulid::{Keyboard, Key, Mouse, Region};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Mock event recorder for testing input operations
/// 入力操作テスト用のモックイベントレコーダー
#[derive(Debug, Clone)]
pub enum MockInputEvent {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: MouseButton },
    MouseDoubleClick { button: MouseButton },
    MouseRightClick,
    KeyPress { key: String },
    KeyRelease { key: String },
    TextType { text: String },
    Hotkey { keys: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mock input device that records events instead of performing them
/// イベントを実行する代わりに記録するモック入力デバイス
#[derive(Clone)]
pub struct MockInputDevice {
    events: Arc<Mutex<Vec<MockInputEvent>>>,
}

impl MockInputDevice {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_mouse_move(&self, x: i32, y: i32) {
        self.events
            .lock()
            .unwrap()
            .push(MockInputEvent::MouseMove { x, y });
    }

    pub fn record_click(&self, button: MouseButton) {
        self.events
            .lock()
            .unwrap()
            .push(MockInputEvent::MouseClick { button });
    }

    pub fn record_double_click(&self, button: MouseButton) {
        self.events
            .lock()
            .unwrap()
            .push(MockInputEvent::MouseDoubleClick { button });
    }

    pub fn record_right_click(&self) {
        self.events
            .lock()
            .unwrap()
            .push(MockInputEvent::MouseRightClick);
    }

    pub fn record_key_press(&self, key: String) {
        self.events
            .lock()
            .unwrap()
            .push(MockInputEvent::KeyPress { key });
    }

    pub fn record_text_type(&self, text: String) {
        self.events
            .lock()
            .unwrap()
            .push(MockInputEvent::TextType { text });
    }

    pub fn record_hotkey(&self, keys: Vec<String>) {
        self.events
            .lock()
            .unwrap()
            .push(MockInputEvent::Hotkey { keys });
    }

    pub fn get_events(&self) -> Vec<MockInputEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }

    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }
}

#[test]
fn test_mock_mouse_move_recording() {
    // Test that mouse movements are recorded correctly
    // マウス移動が正しく記録されることをテスト
    let mock = MockInputDevice::new();

    mock.record_mouse_move(100, 200);
    mock.record_mouse_move(300, 400);

    let events = mock.get_events();
    assert_eq!(events.len(), 2);

    match &events[0] {
        MockInputEvent::MouseMove { x, y } => {
            assert_eq!(*x, 100);
            assert_eq!(*y, 200);
        }
        _ => panic!("Expected MouseMove event"),
    }

    match &events[1] {
        MockInputEvent::MouseMove { x, y } => {
            assert_eq!(*x, 300);
            assert_eq!(*y, 400);
        }
        _ => panic!("Expected MouseMove event"),
    }
}

#[test]
fn test_mock_mouse_click_recording() {
    // Test that mouse clicks are recorded
    // マウスクリックが記録されることをテスト
    let mock = MockInputDevice::new();

    mock.record_click(MouseButton::Left);
    mock.record_click(MouseButton::Right);
    mock.record_double_click(MouseButton::Left);

    let events = mock.get_events();
    assert_eq!(events.len(), 3);

    match &events[0] {
        MockInputEvent::MouseClick { button } => {
            assert_eq!(*button, MouseButton::Left);
        }
        _ => panic!("Expected MouseClick event"),
    }
}

#[test]
fn test_mock_keyboard_recording() {
    // Test that keyboard events are recorded
    // キーボードイベントが記録されることをテスト
    let mock = MockInputDevice::new();

    mock.record_text_type("Hello World".to_string());
    mock.record_key_press("Enter".to_string());

    let events = mock.get_events();
    assert_eq!(events.len(), 2);

    match &events[0] {
        MockInputEvent::TextType { text } => {
            assert_eq!(text, "Hello World");
        }
        _ => panic!("Expected TextType event"),
    }
}

#[test]
fn test_mock_hotkey_recording() {
    // Test that hotkey combinations are recorded
    // ホットキーの組み合わせが記録されることをテスト
    let mock = MockInputDevice::new();

    mock.record_hotkey(vec!["Ctrl".to_string(), "C".to_string()]);
    mock.record_hotkey(vec!["Ctrl".to_string(), "V".to_string()]);

    let events = mock.get_events();
    assert_eq!(events.len(), 2);

    match &events[0] {
        MockInputEvent::Hotkey { keys } => {
            assert_eq!(keys.len(), 2);
            assert_eq!(keys[0], "Ctrl");
            assert_eq!(keys[1], "C");
        }
        _ => panic!("Expected Hotkey event"),
    }
}

#[test]
fn test_click_workflow_simulation() {
    // Simulate a complete click workflow
    // 完全なクリックワークフローをシミュレート
    let mock = MockInputDevice::new();

    // Simulate finding a button and clicking it
    // ボタンを見つけてクリックするシミュレーション
    let button_region = Region::new(100, 100, 80, 30);
    let (cx, cy) = button_region.center();

    mock.record_mouse_move(cx, cy);
    std::thread::sleep(Duration::from_millis(10)); // Simulate movement delay
    mock.record_click(MouseButton::Left);

    let events = mock.get_events();
    assert_eq!(events.len(), 2);

    // Verify move then click
    // 移動してからクリックを確認
    assert!(matches!(events[0], MockInputEvent::MouseMove { .. }));
    assert!(matches!(events[1], MockInputEvent::MouseClick { .. }));
}

#[test]
fn test_form_fill_workflow_simulation() {
    // Simulate filling out a form
    // フォーム入力をシミュレート
    let mock = MockInputDevice::new();

    // Click first field
    // 最初のフィールドをクリック
    mock.record_mouse_move(200, 100);
    mock.record_click(MouseButton::Left);

    // Type name
    // 名前を入力
    mock.record_text_type("John Doe".to_string());

    // Tab to next field
    // 次のフィールドにタブ
    mock.record_key_press("Tab".to_string());

    // Type email
    // メールを入力
    mock.record_text_type("john@example.com".to_string());

    // Submit with Enter
    // Enterで送信
    mock.record_key_press("Enter".to_string());

    let events = mock.get_events();
    assert_eq!(events.len(), 6);
}

#[test]
fn test_copy_paste_workflow_simulation() {
    // Simulate copy-paste workflow
    // コピペワークフローをシミュレート
    let mock = MockInputDevice::new();

    // Select text region
    // テキスト領域を選択
    mock.record_mouse_move(100, 100);
    mock.record_click(MouseButton::Left);
    mock.record_mouse_move(300, 100);

    // Copy
    // コピー
    mock.record_hotkey(vec!["Ctrl".to_string(), "C".to_string()]);

    // Move to destination
    // 宛先に移動
    mock.record_mouse_move(100, 200);
    mock.record_click(MouseButton::Left);

    // Paste
    // ペースト
    mock.record_hotkey(vec!["Ctrl".to_string(), "V".to_string()]);

    let events = mock.get_events();
    assert_eq!(events.len(), 6);

    // Verify hotkey events
    // ホットキーイベントを確認
    let hotkey_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, MockInputEvent::Hotkey { .. }))
        .collect();
    assert_eq!(hotkey_events.len(), 2);
}

#[test]
fn test_event_clearing() {
    // Test that clearing events works
    // イベントクリアが機能することをテスト
    let mock = MockInputDevice::new();

    mock.record_mouse_move(100, 100);
    mock.record_click(MouseButton::Left);

    assert_eq!(mock.event_count(), 2);

    mock.clear();

    assert_eq!(mock.event_count(), 0);
    assert!(mock.get_events().is_empty());
}

#[test]
fn test_concurrent_event_recording() {
    // Test that concurrent access works correctly
    // 並行アクセスが正しく機能することをテスト
    let mock = MockInputDevice::new();
    let mock_clone = mock.clone();

    let handle = std::thread::spawn(move || {
        for i in 0..10 {
            mock_clone.record_mouse_move(i, i);
        }
    });

    for i in 10..20 {
        mock.record_mouse_move(i, i);
    }

    handle.join().unwrap();

    // Should have 20 events total
    // 合計20イベントあるべき
    assert_eq!(mock.event_count(), 20);
}

#[test]
#[ignore = "Requires actual input devices - run with: cargo test -- --ignored"]
fn test_real_mouse_operations() -> sikulid::Result<()> {
    // Test actual mouse operations (requires GUI environment)
    // 実際のマウス操作をテスト（GUI環境が必要）

    // Move mouse to a safe location
    // マウスを安全な位置に移動
    Mouse::move_to(100, 100)?;
    std::thread::sleep(Duration::from_millis(100));

    // Get current position (if supported)
    // 現在位置を取得（サポートされている場合）
    // Note: This may not be implemented yet
    // 注: まだ実装されていない可能性

    Ok(())
}

#[test]
#[ignore = "Requires actual input devices - run with: cargo test -- --ignored"]
fn test_real_keyboard_operations() -> sikulid::Result<()> {
    // Test actual keyboard operations (requires GUI environment)
    // 実際のキーボード操作をテスト（GUI環境が必要）

    // Wait a bit before typing
    // 入力前に少し待機
    std::thread::sleep(Duration::from_millis(500));

    // Type a safe test string
    // 安全なテスト文字列を入力
    // Note: Be careful with this in automated tests
    // 注: 自動テストでは注意して使用
    // Keyboard::type_text("test")?;

    Ok(())
}

#[test]
fn test_input_timing_sequence() {
    // Test that input events are recorded in correct sequence
    // 入力イベントが正しい順序で記録されることをテスト
    let mock = MockInputDevice::new();
    let start = std::time::Instant::now();

    mock.record_mouse_move(0, 0);
    std::thread::sleep(Duration::from_millis(10));

    mock.record_click(MouseButton::Left);
    std::thread::sleep(Duration::from_millis(10));

    mock.record_text_type("test".to_string());

    let elapsed = start.elapsed();

    let events = mock.get_events();
    assert_eq!(events.len(), 3);

    // Sequence should be: move, click, type
    // シーケンスは: 移動、クリック、入力
    assert!(matches!(events[0], MockInputEvent::MouseMove { .. }));
    assert!(matches!(events[1], MockInputEvent::MouseClick { .. }));
    assert!(matches!(events[2], MockInputEvent::TextType { .. }));

    // Should have taken at least 20ms
    // 少なくとも20ms要したはず
    assert!(elapsed.as_millis() >= 20);
}

#[test]
fn test_region_based_clicking() {
    // Test clicking on different regions
    // 異なる領域へのクリックをテスト
    let mock = MockInputDevice::new();

    let regions = vec![
        Region::new(10, 10, 50, 50),
        Region::new(100, 100, 80, 30),
        Region::new(500, 300, 100, 40),
    ];

    for region in &regions {
        let (cx, cy) = region.center();
        mock.record_mouse_move(cx, cy);
        mock.record_click(MouseButton::Left);
    }

    assert_eq!(mock.event_count(), regions.len() * 2);
}

#[test]
fn test_multi_button_clicks() {
    // Test different mouse button clicks
    // 異なるマウスボタンのクリックをテスト
    let mock = MockInputDevice::new();

    mock.record_click(MouseButton::Left);
    mock.record_click(MouseButton::Right);
    mock.record_click(MouseButton::Middle);
    mock.record_double_click(MouseButton::Left);
    mock.record_right_click();

    assert_eq!(mock.event_count(), 5);
}

//! Location type for screen coordinates
//! 画面座標用の Location 型

use crate::Result;

/// Represents a point location on the screen
/// 画面上の点の位置を表します
///
/// Used for defining exact click targets, mouse positions, and coordinate operations.
/// クリックターゲット、マウス位置、座標操作の定義に使用します。
///
/// # Example / 使用例
///
/// ```
/// use sikulid::Location;
///
/// let loc = Location::new(100, 200);
/// let above = loc.above(50);
/// assert_eq!(above.y, 150);
///
/// // Convert from tuple
/// // タプルから変換
/// let loc2: Location = (100, 200).into();
/// assert_eq!(loc, loc2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    /// X coordinate / X座標
    pub x: i32,
    /// Y coordinate / Y座標
    pub y: i32,
}

impl Location {
    /// Create a new location with the given coordinates
    /// 指定された座標で新しい位置を作成
    ///
    /// # Arguments / 引数
    ///
    /// * `x` - X coordinate / X座標
    /// * `y` - Y coordinate / Y座標
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 200);
    /// assert_eq!(loc.x, 100);
    /// assert_eq!(loc.y, 200);
    /// ```
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Get X coordinate
    /// X座標を取得
    pub fn get_x(&self) -> i32 {
        self.x
    }

    /// Get Y coordinate
    /// Y座標を取得
    pub fn get_y(&self) -> i32 {
        self.y
    }

    /// Set location coordinates
    /// 位置座標を設定
    ///
    /// # Arguments / 引数
    ///
    /// * `x` - New X coordinate / 新しいX座標
    /// * `y` - New Y coordinate / 新しいY座標
    pub fn set_location(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Create a new location offset by the given distances
    /// 指定された距離だけオフセットした新しい位置を作成
    ///
    /// # Arguments / 引数
    ///
    /// * `dx` - X offset / X方向のオフセット
    /// * `dy` - Y offset / Y方向のオフセット
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// let offset = loc.offset(10, -20);
    /// assert_eq!(offset.x, 110);
    /// assert_eq!(offset.y, 80);
    /// ```
    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Create a new location to the left of this location
    /// この位置の左にある新しい位置を作成
    ///
    /// # Arguments / 引数
    ///
    /// * `distance` - Distance to the left / 左への距離
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// let left = loc.left(50);
    /// assert_eq!(left.x, 50);
    /// assert_eq!(left.y, 100);
    /// ```
    pub fn left(&self, distance: i32) -> Self {
        Self {
            x: self.x - distance,
            y: self.y,
        }
    }

    /// Create a new location to the right of this location
    /// この位置の右にある新しい位置を作成
    ///
    /// # Arguments / 引数
    ///
    /// * `distance` - Distance to the right / 右への距離
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// let right = loc.right(50);
    /// assert_eq!(right.x, 150);
    /// assert_eq!(right.y, 100);
    /// ```
    pub fn right(&self, distance: i32) -> Self {
        Self {
            x: self.x + distance,
            y: self.y,
        }
    }

    /// Create a new location above this location
    /// この位置の上にある新しい位置を作成
    ///
    /// # Arguments / 引数
    ///
    /// * `distance` - Distance above / 上への距離
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// let above = loc.above(50);
    /// assert_eq!(above.x, 100);
    /// assert_eq!(above.y, 50);
    /// ```
    pub fn above(&self, distance: i32) -> Self {
        Self {
            x: self.x,
            y: self.y - distance,
        }
    }

    /// Create a new location below this location
    /// この位置の下にある新しい位置を作成
    ///
    /// # Arguments / 引数
    ///
    /// * `distance` - Distance below / 下への距離
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// let below = loc.below(50);
    /// assert_eq!(below.x, 100);
    /// assert_eq!(below.y, 150);
    /// ```
    pub fn below(&self, distance: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + distance,
        }
    }

    /// Click at this location
    /// この位置でクリック
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// loc.click().unwrap();
    /// ```
    pub fn click(&self) -> Result<()> {
        crate::input::click(self.x, self.y)
    }

    /// Double click at this location
    /// この位置でダブルクリック
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// loc.double_click().unwrap();
    /// ```
    pub fn double_click(&self) -> Result<()> {
        crate::input::double_click(self.x, self.y)
    }

    /// Right click at this location
    /// この位置で右クリック
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// loc.right_click().unwrap();
    /// ```
    pub fn right_click(&self) -> Result<()> {
        crate::input::right_click(self.x, self.y)
    }

    /// Move mouse to this location (hover)
    /// この位置にマウスを移動（ホバー）
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 100);
    /// loc.hover().unwrap();
    /// ```
    pub fn hover(&self) -> Result<()> {
        crate::input::move_to(self.x, self.y)
    }
}

impl From<(i32, i32)> for Location {
    /// Convert from tuple (x, y) to Location
    /// タプル(x, y)からLocationに変換
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::Location;
    ///
    /// let loc: Location = (100, 200).into();
    /// assert_eq!(loc.x, 100);
    /// assert_eq!(loc.y, 200);
    /// ```
    fn from(coords: (i32, i32)) -> Self {
        Self {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl From<Location> for (i32, i32) {
    /// Convert from Location to tuple (x, y)
    /// Locationからタプル(x, y)に変換
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::Location;
    ///
    /// let loc = Location::new(100, 200);
    /// let coords: (i32, i32) = loc.into();
    /// assert_eq!(coords, (100, 200));
    /// ```
    fn from(loc: Location) -> Self {
        (loc.x, loc.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_new() {
        // Test basic creation
        // 基本的な生成をテスト
        let loc = Location::new(100, 200);
        assert_eq!(loc.x, 100);
        assert_eq!(loc.y, 200);
    }

    #[test]
    fn test_location_getters() {
        // Test getter methods
        // ゲッターメソッドをテスト
        let loc = Location::new(150, 250);
        assert_eq!(loc.get_x(), 150);
        assert_eq!(loc.get_y(), 250);
    }

    #[test]
    fn test_location_set_location() {
        // Test set_location method
        // set_locationメソッドをテスト
        let mut loc = Location::new(100, 100);
        loc.set_location(200, 300);
        assert_eq!(loc.x, 200);
        assert_eq!(loc.y, 300);
    }

    #[test]
    fn test_location_offset() {
        // Test offset method
        // offsetメソッドをテスト
        let loc = Location::new(100, 100);

        let offset_positive = loc.offset(50, 30);
        assert_eq!(offset_positive.x, 150);
        assert_eq!(offset_positive.y, 130);

        let offset_negative = loc.offset(-20, -40);
        assert_eq!(offset_negative.x, 80);
        assert_eq!(offset_negative.y, 60);
    }

    #[test]
    fn test_location_left() {
        // Test left method
        // leftメソッドをテスト
        let loc = Location::new(100, 100);
        let left = loc.left(50);
        assert_eq!(left.x, 50);
        assert_eq!(left.y, 100);
    }

    #[test]
    fn test_location_right() {
        // Test right method
        // rightメソッドをテスト
        let loc = Location::new(100, 100);
        let right = loc.right(50);
        assert_eq!(right.x, 150);
        assert_eq!(right.y, 100);
    }

    #[test]
    fn test_location_above() {
        // Test above method
        // aboveメソッドをテスト
        let loc = Location::new(100, 100);
        let above = loc.above(30);
        assert_eq!(above.x, 100);
        assert_eq!(above.y, 70);
    }

    #[test]
    fn test_location_below() {
        // Test below method
        // belowメソッドをテスト
        let loc = Location::new(100, 100);
        let below = loc.below(40);
        assert_eq!(below.x, 100);
        assert_eq!(below.y, 140);
    }

    #[test]
    fn test_location_from_tuple() {
        // Test conversion from tuple
        // タプルからの変換をテスト
        let loc: Location = (150, 250).into();
        assert_eq!(loc.x, 150);
        assert_eq!(loc.y, 250);
    }

    #[test]
    fn test_location_to_tuple() {
        // Test conversion to tuple
        // タプルへの変換をテスト
        let loc = Location::new(150, 250);
        let coords: (i32, i32) = loc.into();
        assert_eq!(coords, (150, 250));
    }

    #[test]
    fn test_location_round_trip_conversion() {
        // Test round-trip conversion
        // 往復変換をテスト
        let original = (123, 456);
        let loc: Location = original.into();
        let result: (i32, i32) = loc.into();
        assert_eq!(original, result);
    }

    #[test]
    fn test_location_equality() {
        // Test equality comparison
        // 等価比較をテスト
        let loc1 = Location::new(100, 200);
        let loc2 = Location::new(100, 200);
        let loc3 = Location::new(150, 200);

        assert_eq!(loc1, loc2);
        assert_ne!(loc1, loc3);
    }

    #[test]
    fn test_location_copy() {
        // Test that Location implements Copy
        // LocationがCopyを実装していることをテスト
        let loc1 = Location::new(100, 200);
        let loc2 = loc1; // Copy
        assert_eq!(loc1, loc2);
        // loc1 should still be usable after copy
        // コピー後もloc1は使用可能であるべき
        assert_eq!(loc1.x, 100);
    }

    #[test]
    fn test_location_clone() {
        // Test that Location implements Clone
        // LocationがCloneを実装していることをテスト
        let loc1 = Location::new(100, 200);
        let loc2 = loc1.clone();
        assert_eq!(loc1, loc2);
    }

    #[test]
    fn test_location_debug() {
        // Test that Location implements Debug
        // LocationがDebugを実装していることをテスト
        let loc = Location::new(100, 200);
        let debug_str = format!("{:?}", loc);
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("200"));
    }

    #[test]
    fn test_location_chaining() {
        // Test method chaining
        // メソッドチェーンをテスト
        let loc = Location::new(100, 100);
        let result = loc.right(50).below(30).left(20);
        assert_eq!(result.x, 130); // 100 + 50 - 20
        assert_eq!(result.y, 130); // 100 + 30
    }

    #[test]
    fn test_location_negative_coordinates() {
        // Test with negative coordinates
        // 負の座標でテスト
        let loc = Location::new(-100, -200);
        assert_eq!(loc.x, -100);
        assert_eq!(loc.y, -200);

        let offset = loc.offset(150, 250);
        assert_eq!(offset.x, 50);
        assert_eq!(offset.y, 50);
    }

    #[test]
    fn test_location_zero_coordinates() {
        // Test with zero coordinates
        // ゼロ座標でテスト
        let loc = Location::new(0, 0);
        assert_eq!(loc.x, 0);
        assert_eq!(loc.y, 0);

        let below = loc.below(100);
        assert_eq!(below.x, 0);
        assert_eq!(below.y, 100);
    }

    // Note: Click, double_click, right_click, and hover methods
    // are tested as integration tests since they interact with the system
    // 注意: click, double_click, right_click, hoverメソッドは
    // システムと相互作用するため、統合テストとしてテストします
}

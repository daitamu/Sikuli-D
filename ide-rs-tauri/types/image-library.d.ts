/**
 * Type definitions for Image Library API
 * 画像ライブラリAPI型定義
 */

declare module '@tauri-apps/api/image-library' {
  /**
   * Image metadata and information
   * 画像メタデータと情報
   */
  export interface ImageInfo {
    /** Full file path / 完全ファイルパス */
    path: string;

    /** File name only / ファイル名のみ */
    name: string;

    /** Image width in pixels / 画像幅（ピクセル） */
    width: number;

    /** Image height in pixels / 画像高さ（ピクセル） */
    height: number;

    /** File size in bytes / ファイルサイズ（バイト） */
    file_size: number;

    /** Creation timestamp (ISO 8601) / 作成タイムスタンプ（ISO 8601） */
    created_at: string;

    /** Base64-encoded thumbnail (optional) / Base64エンコードされたサムネイル（オプション） */
    thumbnail?: string;

    /** Number of times referenced in scripts / スクリプト内での参照回数 */
    usage_count: number;
  }

  /**
   * List all images in a project bundle
   * プロジェクトバンドル内のすべての画像をリスト
   *
   * @param projectPath - Path to .sikuli bundle
   * @returns Array of image information
   *
   * @example
   * ```typescript
   * const images = await invoke<ImageInfo[]>('list_images_command', {
   *   projectPath: '/path/to/project.sikuli'
   * });
   * ```
   */
  export function list_images_command(args: {
    projectPath: string;
  }): Promise<ImageInfo[]>;

  /**
   * Get image thumbnail as Base64 string
   * 画像サムネイルをBase64文字列として取得
   *
   * @param path - Path to image file
   * @param size - Maximum thumbnail size (width/height)
   * @returns Base64-encoded PNG thumbnail
   *
   * @example
   * ```typescript
   * const thumbnail = await invoke<string>('get_image_thumbnail_command', {
   *   path: '/path/to/image.png',
   *   size: 128
   * });
   * // Use as: `data:image/png;base64,${thumbnail}`
   * ```
   */
  export function get_image_thumbnail_command(args: {
    path: string;
    size: number;
  }): Promise<string>;

  /**
   * Delete an image file
   * 画像ファイルを削除
   *
   * @param path - Path to image file to delete
   *
   * @example
   * ```typescript
   * await invoke('delete_image_command', {
   *   path: '/path/to/image.png'
   * });
   * ```
   */
  export function delete_image_command(args: { path: string }): Promise<void>;

  /**
   * Rename an image file
   * 画像ファイルの名前を変更
   *
   * @param oldPath - Current image path
   * @param newName - New file name (without directory)
   * @returns New full path
   *
   * @example
   * ```typescript
   * const newPath = await invoke<string>('rename_image_command', {
   *   oldPath: '/path/to/old.png',
   *   newName: 'new.png'
   * });
   * ```
   */
  export function rename_image_command(args: {
    oldPath: string;
    newName: string;
  }): Promise<string>;

  /**
   * Find unused images in a project
   * プロジェクト内の未使用画像を検索
   *
   * @param projectPath - Path to .sikuli bundle
   * @returns Array of unused image paths
   *
   * @example
   * ```typescript
   * const unused = await invoke<string[]>('find_unused_images_command', {
   *   projectPath: '/path/to/project.sikuli'
   * });
   * console.log(`Found ${unused.length} unused images`);
   * ```
   */
  export function find_unused_images_command(args: {
    projectPath: string;
  }): Promise<string[]>;

  /**
   * Import images to a project bundle
   * プロジェクトバンドルに画像をインポート
   *
   * @param paths - Array of image paths to import
   * @param projectPath - Target .sikuli bundle path
   * @returns Array of imported image paths in bundle
   *
   * @example
   * ```typescript
   * const imported = await invoke<string[]>('import_images_command', {
   *   paths: ['/path/to/image1.png', '/path/to/image2.png'],
   *   projectPath: '/path/to/project.sikuli'
   * });
   * console.log(`Imported ${imported.length} images`);
   * ```
   */
  export function import_images_command(args: {
    paths: string[];
    projectPath: string;
  }): Promise<string[]>;
}

// Re-export for convenience
export { ImageInfo } from '@tauri-apps/api/image-library';

/**
 * Image Library Component / 画像ライブラリコンポーネント
 *
 * Manages images within .sikuli bundles
 * .sikuliバンドル内の画像を管理します
 */

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Type Definitions / 型定義
// ============================================================================

/**
 * Image metadata structure
 * 画像メタデータ構造
 */
interface ImageInfo {
  path: string;
  name: string;
  width: number;
  height: number;
  file_size: number;
  created_at: string;
  thumbnail?: string;
  usage_count: number;
}

/**
 * View mode for image display
 * 画像表示のビューモード
 */
type ViewMode = 'grid' | 'list';

/**
 * Sort field for images
 * 画像のソートフィールド
 */
type SortField = 'name' | 'size' | 'date' | 'usage';

// ============================================================================
// Main Component / メインコンポーネント
// ============================================================================

interface ImageLibraryProps {
  projectPath: string;
  onImageSelect?: (imagePath: string) => void;
  onImageInsert?: (imagePath: string) => void;
}

export const ImageLibrary: React.FC<ImageLibraryProps> = ({
  projectPath,
  onImageSelect,
  onImageInsert,
}) => {
  const [images, setImages] = useState<ImageInfo[]>([]);
  const [filteredImages, setFilteredImages] = useState<ImageInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedImage, setSelectedImage] = useState<string | null>(null);
  const [showUnusedOnly, setShowUnusedOnly] = useState(false);
  const [sortField, setSortField] = useState<SortField>('name');
  const [sortAscending, setSortAscending] = useState(true);

  // ============================================================================
  // Data Loading / データ読み込み
  // ============================================================================

  /**
   * Load images from project
   * プロジェクトから画像を読み込む
   */
  const loadImages = useCallback(async () => {
    if (!projectPath) return;

    setLoading(true);
    setError(null);

    try {
      const result = await invoke<ImageInfo[]>('list_images_command', {
        projectPath,
      });

      setImages(result);
      setFilteredImages(result);
    } catch (err) {
      const errorMsg = typeof err === 'string' ? err : String(err);
      setError(errorMsg);
      console.error('Failed to load images:', err);
    } finally {
      setLoading(false);
    }
  }, [projectPath]);

  /**
   * Load thumbnail for an image
   * 画像のサムネイルを読み込む
   */
  const loadThumbnail = useCallback(async (imagePath: string, size: number = 128) => {
    try {
      const thumbnail = await invoke<string>('get_image_thumbnail_command', {
        path: imagePath,
        size,
      });

      // Update image in state with thumbnail
      setImages((prev) =>
        prev.map((img) =>
          img.path === imagePath ? { ...img, thumbnail: `data:image/png;base64,${thumbnail}` } : img
        )
      );
    } catch (err) {
      console.error('Failed to load thumbnail:', err);
    }
  }, []);

  // Load images on mount and when project path changes
  useEffect(() => {
    loadImages();
  }, [loadImages]);

  // Load thumbnails for visible images
  useEffect(() => {
    const imagesToLoad = filteredImages.filter((img) => !img.thumbnail).slice(0, 20);

    imagesToLoad.forEach((img) => {
      loadThumbnail(img.path);
    });
  }, [filteredImages, loadThumbnail]);

  // ============================================================================
  // Filtering and Sorting / フィルタリングとソート
  // ============================================================================

  /**
   * Apply search filter and sorting
   * 検索フィルターとソートを適用
   */
  useEffect(() => {
    let filtered = [...images];

    // Apply search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter((img) => img.name.toLowerCase().includes(query));
    }

    // Apply unused filter
    if (showUnusedOnly) {
      filtered = filtered.filter((img) => img.usage_count === 0);
    }

    // Apply sorting
    filtered.sort((a, b) => {
      let comparison = 0;

      switch (sortField) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'size':
          comparison = a.file_size - b.file_size;
          break;
        case 'date':
          comparison = new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
          break;
        case 'usage':
          comparison = a.usage_count - b.usage_count;
          break;
      }

      return sortAscending ? comparison : -comparison;
    });

    setFilteredImages(filtered);
  }, [images, searchQuery, showUnusedOnly, sortField, sortAscending]);

  // ============================================================================
  // Image Actions / 画像操作
  // ============================================================================

  /**
   * Delete an image
   * 画像を削除
   */
  const handleDelete = useCallback(
    async (imagePath: string) => {
      if (!confirm(`Delete ${imagePath.split(/[\\/]/).pop()}?`)) {
        return;
      }

      try {
        await invoke('delete_image_command', { path: imagePath });
        await loadImages();
      } catch (err) {
        alert(`Failed to delete image: ${err}`);
      }
    },
    [loadImages]
  );

  /**
   * Rename an image
   * 画像の名前を変更
   */
  const handleRename = useCallback(
    async (imagePath: string) => {
      const currentName = imagePath.split(/[\\/]/).pop() || '';
      const newName = prompt('Enter new name:', currentName);

      if (!newName || newName === currentName) {
        return;
      }

      try {
        await invoke('rename_image_command', {
          oldPath: imagePath,
          newName,
        });
        await loadImages();
      } catch (err) {
        alert(`Failed to rename image: ${err}`);
      }
    },
    [loadImages]
  );

  /**
   * Import images from file system
   * ファイルシステムから画像をインポート
   */
  const handleImport = useCallback(async () => {
    // This would typically use Tauri's file dialog
    // For now, show a message
    alert('Image import via file dialog will be implemented with tauri-plugin-dialog');
  }, []);

  /**
   * Find and highlight unused images
   * 未使用画像を検索してハイライト
   */
  const handleFindUnused = useCallback(async () => {
    try {
      const unused = await invoke<string[]>('find_unused_images_command', {
        projectPath,
      });

      if (unused.length === 0) {
        alert('No unused images found!');
      } else {
        alert(`Found ${unused.length} unused images`);
        setShowUnusedOnly(true);
      }
    } catch (err) {
      alert(`Failed to find unused images: ${err}`);
    }
  }, [projectPath]);

  // ============================================================================
  // Render Helpers / レンダリングヘルパー
  // ============================================================================

  /**
   * Format file size for display
   * ファイルサイズを表示用にフォーマット
   */
  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  /**
   * Format date for display
   * 日付を表示用にフォーマット
   */
  const formatDate = (isoDate: string): string => {
    return new Date(isoDate).toLocaleString();
  };

  // ============================================================================
  // Render / レンダリング
  // ============================================================================

  return (
    <div className="image-library" style={styles.container}>
      {/* Toolbar / ツールバー */}
      <div style={styles.toolbar}>
        <input
          type="text"
          placeholder="🔍 Search images..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          style={styles.searchInput}
        />

        <select
          value={viewMode}
          onChange={(e) => setViewMode(e.target.value as ViewMode)}
          style={styles.select}
        >
          <option value="grid">Grid View</option>
          <option value="list">List View</option>
        </select>

        <select
          value={sortField}
          onChange={(e) => setSortField(e.target.value as SortField)}
          style={styles.select}
        >
          <option value="name">Sort by Name</option>
          <option value="size">Sort by Size</option>
          <option value="date">Sort by Date</option>
          <option value="usage">Sort by Usage</option>
        </select>

        <button onClick={() => setSortAscending(!sortAscending)} style={styles.button}>
          {sortAscending ? '↑' : '↓'}
        </button>

        <button onClick={handleImport} style={styles.button}>
          + Import
        </button>

        <button onClick={handleFindUnused} style={styles.button}>
          🔍 Find Unused
        </button>

        <label style={styles.checkbox}>
          <input
            type="checkbox"
            checked={showUnusedOnly}
            onChange={(e) => setShowUnusedOnly(e.target.checked)}
          />
          Show Unused Only
        </label>

        <button onClick={loadImages} style={styles.button} disabled={loading}>
          🔄 Refresh
        </button>
      </div>

      {/* Error Display / エラー表示 */}
      {error && (
        <div style={styles.error}>
          Error: {error}
        </div>
      )}

      {/* Loading State / 読み込み中 */}
      {loading && (
        <div style={styles.loading}>
          Loading images...
        </div>
      )}

      {/* Image Display / 画像表示 */}
      {!loading && !error && (
        <>
          {viewMode === 'grid' ? (
            <div style={styles.gridContainer}>
              {filteredImages.map((image) => (
                <ImageCard
                  key={image.path}
                  image={image}
                  selected={selectedImage === image.path}
                  onClick={() => {
                    setSelectedImage(image.path);
                    onImageSelect?.(image.path);
                  }}
                  onDelete={() => handleDelete(image.path)}
                  onRename={() => handleRename(image.path)}
                  onInsert={() => onImageInsert?.(image.path)}
                />
              ))}
            </div>
          ) : (
            <div style={styles.listContainer}>
              {filteredImages.map((image) => (
                <ImageListItem
                  key={image.path}
                  image={image}
                  selected={selectedImage === image.path}
                  onClick={() => {
                    setSelectedImage(image.path);
                    onImageSelect?.(image.path);
                  }}
                  onDelete={() => handleDelete(image.path)}
                  onRename={() => handleRename(image.path)}
                  onInsert={() => onImageInsert?.(image.path)}
                  formatFileSize={formatFileSize}
                  formatDate={formatDate}
                />
              ))}
            </div>
          )}
        </>
      )}

      {/* Status Bar / ステータスバー */}
      <div style={styles.statusBar}>
        <span>{filteredImages.length} images</span>
        <span>|</span>
        <span>
          {filteredImages.filter((img) => img.usage_count === 0).length} unused
        </span>
        <span>|</span>
        <span>
          {formatFileSize(filteredImages.reduce((sum, img) => sum + img.file_size, 0))} total
        </span>
      </div>
    </div>
  );
};

// ============================================================================
// Sub-Components / サブコンポーネント
// ============================================================================

/**
 * Image card for grid view
 * グリッドビュー用画像カード
 */
interface ImageCardProps {
  image: ImageInfo;
  selected: boolean;
  onClick: () => void;
  onDelete: () => void;
  onRename: () => void;
  onInsert: () => void;
}

const ImageCard: React.FC<ImageCardProps> = ({
  image,
  selected,
  onClick,
  onDelete,
  onRename,
  onInsert,
}) => {
  const [showMenu, setShowMenu] = useState(false);

  return (
    <div
      style={{
        ...styles.imageCard,
        ...(selected ? styles.imageCardSelected : {}),
      }}
      onClick={onClick}
      onContextMenu={(e) => {
        e.preventDefault();
        setShowMenu(!showMenu);
      }}
    >
      {/* Thumbnail / サムネイル */}
      <div style={styles.thumbnailContainer}>
        {image.thumbnail ? (
          <img src={image.thumbnail} alt={image.name} style={styles.thumbnail} />
        ) : (
          <div style={styles.thumbnailPlaceholder}>🖼️</div>
        )}
      </div>

      {/* Image Info / 画像情報 */}
      <div style={styles.imageInfo}>
        <div style={styles.imageName} title={image.name}>
          {image.name}
        </div>
        <div style={styles.imageMeta}>
          {image.width} × {image.height}
        </div>
        {image.usage_count === 0 && (
          <div style={styles.unusedBadge}>Unused</div>
        )}
      </div>

      {/* Context Menu / コンテキストメニュー */}
      {showMenu && (
        <div style={styles.contextMenu} onMouseLeave={() => setShowMenu(false)}>
          <button onClick={onInsert} style={styles.menuItem}>
            Insert to Editor
          </button>
          <button onClick={onRename} style={styles.menuItem}>
            Rename
          </button>
          <button onClick={onDelete} style={styles.menuItem}>
            Delete
          </button>
        </div>
      )}
    </div>
  );
};

/**
 * Image item for list view
 * リストビュー用画像アイテム
 */
interface ImageListItemProps {
  image: ImageInfo;
  selected: boolean;
  onClick: () => void;
  onDelete: () => void;
  onRename: () => void;
  onInsert: () => void;
  formatFileSize: (bytes: number) => string;
  formatDate: (date: string) => string;
}

const ImageListItem: React.FC<ImageListItemProps> = ({
  image,
  selected,
  onClick,
  onDelete,
  onRename,
  onInsert,
  formatFileSize,
  formatDate,
}) => {
  return (
    <div
      style={{
        ...styles.listItem,
        ...(selected ? styles.listItemSelected : {}),
      }}
      onClick={onClick}
    >
      <div style={styles.listItemThumbnail}>
        {image.thumbnail ? (
          <img src={image.thumbnail} alt={image.name} style={styles.listThumbnail} />
        ) : (
          <div style={styles.listThumbnailPlaceholder}>🖼️</div>
        )}
      </div>

      <div style={styles.listItemName}>{image.name}</div>
      <div style={styles.listItemSize}>{formatFileSize(image.file_size)}</div>
      <div style={styles.listItemDimensions}>
        {image.width} × {image.height}
      </div>
      <div style={styles.listItemDate}>{formatDate(image.created_at)}</div>
      <div style={styles.listItemUsage}>
        {image.usage_count} reference{image.usage_count !== 1 ? 's' : ''}
      </div>

      <div style={styles.listItemActions}>
        <button onClick={onInsert} style={styles.actionButton}>
          Insert
        </button>
        <button onClick={onRename} style={styles.actionButton}>
          Rename
        </button>
        <button onClick={onDelete} style={styles.actionButton}>
          Delete
        </button>
      </div>
    </div>
  );
};

// ============================================================================
// Styles / スタイル
// ============================================================================

const styles: { [key: string]: React.CSSProperties } = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#1e1e1e',
    color: '#d4d4d4',
  },
  toolbar: {
    display: 'flex',
    gap: '8px',
    padding: '12px',
    backgroundColor: '#2d2d2d',
    borderBottom: '1px solid #3e3e3e',
    alignItems: 'center',
  },
  searchInput: {
    flex: 1,
    padding: '8px 12px',
    backgroundColor: '#3c3c3c',
    border: '1px solid #555',
    borderRadius: '4px',
    color: '#d4d4d4',
    fontSize: '14px',
  },
  select: {
    padding: '8px 12px',
    backgroundColor: '#3c3c3c',
    border: '1px solid #555',
    borderRadius: '4px',
    color: '#d4d4d4',
    fontSize: '14px',
  },
  button: {
    padding: '8px 16px',
    backgroundColor: '#0e639c',
    border: 'none',
    borderRadius: '4px',
    color: '#fff',
    fontSize: '14px',
    cursor: 'pointer',
  },
  checkbox: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '14px',
  },
  error: {
    padding: '16px',
    backgroundColor: '#5a1d1d',
    color: '#f48771',
    borderBottom: '1px solid #8b3434',
  },
  loading: {
    padding: '32px',
    textAlign: 'center',
    fontSize: '16px',
    color: '#888',
  },
  gridContainer: {
    flex: 1,
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))',
    gap: '16px',
    padding: '16px',
    overflowY: 'auto',
  },
  listContainer: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    overflowY: 'auto',
  },
  imageCard: {
    position: 'relative',
    display: 'flex',
    flexDirection: 'column',
    padding: '12px',
    backgroundColor: '#2d2d2d',
    border: '2px solid transparent',
    borderRadius: '8px',
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  imageCardSelected: {
    borderColor: '#0e639c',
    backgroundColor: '#3c3c3c',
  },
  thumbnailContainer: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    height: '100px',
    marginBottom: '8px',
  },
  thumbnail: {
    maxWidth: '100%',
    maxHeight: '100%',
    objectFit: 'contain',
  },
  thumbnailPlaceholder: {
    fontSize: '48px',
    opacity: 0.3,
  },
  imageInfo: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
  },
  imageName: {
    fontSize: '14px',
    fontWeight: 'bold',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  imageMeta: {
    fontSize: '12px',
    color: '#888',
  },
  unusedBadge: {
    fontSize: '11px',
    padding: '2px 6px',
    backgroundColor: '#f48771',
    color: '#1e1e1e',
    borderRadius: '4px',
    alignSelf: 'flex-start',
  },
  contextMenu: {
    position: 'absolute',
    top: '8px',
    right: '8px',
    display: 'flex',
    flexDirection: 'column',
    backgroundColor: '#2d2d2d',
    border: '1px solid #555',
    borderRadius: '4px',
    boxShadow: '0 4px 8px rgba(0, 0, 0, 0.3)',
    zIndex: 10,
  },
  menuItem: {
    padding: '8px 16px',
    backgroundColor: 'transparent',
    border: 'none',
    color: '#d4d4d4',
    textAlign: 'left',
    cursor: 'pointer',
    fontSize: '14px',
  },
  listItem: {
    display: 'grid',
    gridTemplateColumns: '60px 1fr 100px 120px 140px 120px 180px',
    gap: '12px',
    padding: '12px 16px',
    backgroundColor: '#2d2d2d',
    borderBottom: '1px solid #3e3e3e',
    alignItems: 'center',
    cursor: 'pointer',
  },
  listItemSelected: {
    backgroundColor: '#3c3c3c',
    borderLeft: '3px solid #0e639c',
  },
  listItemThumbnail: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
  },
  listThumbnail: {
    width: '40px',
    height: '40px',
    objectFit: 'contain',
  },
  listThumbnailPlaceholder: {
    fontSize: '24px',
    opacity: 0.3,
  },
  listItemName: {
    fontSize: '14px',
    fontWeight: 'bold',
  },
  listItemSize: {
    fontSize: '13px',
    color: '#888',
  },
  listItemDimensions: {
    fontSize: '13px',
    color: '#888',
  },
  listItemDate: {
    fontSize: '13px',
    color: '#888',
  },
  listItemUsage: {
    fontSize: '13px',
    color: '#888',
  },
  listItemActions: {
    display: 'flex',
    gap: '8px',
  },
  actionButton: {
    padding: '6px 12px',
    backgroundColor: '#0e639c',
    border: 'none',
    borderRadius: '4px',
    color: '#fff',
    fontSize: '12px',
    cursor: 'pointer',
  },
  statusBar: {
    display: 'flex',
    gap: '12px',
    padding: '12px 16px',
    backgroundColor: '#252526',
    borderTop: '1px solid #3e3e3e',
    fontSize: '13px',
    color: '#888',
  },
};

export default ImageLibrary;

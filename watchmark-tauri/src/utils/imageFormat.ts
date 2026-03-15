import { convertFileSrc } from '@tauri-apps/api/core';

/**
 * Formats an image path returned from the Rust backend.
 *
 * If the path is an absolute local file path (e.g., C:\... or /home/...),
 * it converts it to a Tauri asset:// URL.
 *
 * If it is a raw TMDB path (e.g., /xyz.jpg), it prepends the TMDB base URL.
 *
 * @param path The image path from the backend.
 * @param size The TMDB image size (e.g., 'w500', 'original'). Defaults to 'w500'.
 * @returns The formatted, ready-to-use image URL.
 */
export function formatImagePath(path: string | null | undefined, size: string = 'w500'): string {
  if (!path || path.trim() === '') {
    return '';
  }

  // Double Conversion Protection
  if (path.startsWith('asset.localhost') || path.startsWith('http') || path.startsWith('asset://')) {
    return path;
  }

  // Check if it's an absolute local path
  // Windows: starts with a drive letter (e.g., C:\ or C:/) or a network share (\\)
  // Unix/Linux/macOS: starts with / and contains another / (to distinguish from TMDB paths like /xyz.jpg)
  const isWindowsPath = /^[a-zA-Z]:[\\/]|^\/\/[a-zA-Z0-9]|^\\\\[a-zA-Z0-9]/.test(path);
  const isUnixPath = path.startsWith('/') && path.indexOf('/', 1) !== -1;

  if (isWindowsPath || isUnixPath) {
    // Normalize backslashes to forward slashes for Tauri v2 Windows compatibility
    // This prevents 403 Forbidden errors caused by double-encoded '%5C' in URI schemes.
    const normalizedPath = path.replace(/\\/g, '/');
    // It's a local cached file, use Tauri's custom protocol
    return convertFileSrc(normalizedPath);
  }

  // Otherwise, it's a raw TMDB path
  // Ensure it starts with a slash
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `https://image.tmdb.org/t/p/${size}${cleanPath}`;
}

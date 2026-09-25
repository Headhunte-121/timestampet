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

  // 1. Remove Windows Verbatim prefix (\\?\ or \??\) if present first
  let cleanPath = path.trim().replace(/^\\\\\?\\/, '').replace(/^\\\?\?\\/, '');

  // 2. Double Conversion & Protocol Protection
  if (
    cleanPath.startsWith('asset.localhost') ||
    cleanPath.startsWith('http://') ||
    cleanPath.startsWith('https://') ||
    cleanPath.startsWith('asset://') ||
    cleanPath.startsWith('watchmark://') ||
    cleanPath.startsWith('data:') ||
    cleanPath.startsWith('blob:')
  ) {
    return cleanPath;
  }

  // 3. Check if it's an absolute local path
  // Windows: starts with a drive letter (e.g., C:\ or C:/) or a network share (\\ or //)
  // Unix/Linux/macOS: starts with / and contains another / (to distinguish from TMDB paths like /xyz.jpg)
  const isWindowsPath = /^[a-zA-Z]:[\\/]|^\\\\|^\/\//.test(cleanPath);
  const isUnixPath = cleanPath.startsWith('/') && cleanPath.indexOf('/', 1) !== -1;

  if (isWindowsPath || isUnixPath) {
    // Force ALL backslashes to forward slashes for URI compatibility
    // This prevents 403 Forbidden errors caused by double-encoded '%5C' in URI schemes.
    cleanPath = cleanPath.replace(/\\/g, '/');

    // Convert to Tauri Asset URL (e.g. http://asset.localhost/...)
    return convertFileSrc(cleanPath);
  }

  // 4. Otherwise, it's a raw TMDB path
  // Ensure it starts with a slash
  const tmdbPath = cleanPath.startsWith('/') ? cleanPath : `/${cleanPath}`;
  return `https://image.tmdb.org/t/p/${size}${tmdbPath}`;
}

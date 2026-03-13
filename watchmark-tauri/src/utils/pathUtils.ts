export function formatWindowsPath(path: string): string {
  if (!path) return path;

  // Aggressively replace all forward slashes with backslashes
  let formatted = path.replace(/\//g, '\\');

  // Handle multiple consecutive slashes (e.g., C:\\\\Movies -> C:\Movies)
  // Be careful not to replace network shares (e.g., \\Server\Share)
  // We can reduce any sequence of backslashes greater than 2 to a single backslash
  // Then we can check if it starts with \\ for network drives.

  if (formatted.startsWith('\\\\')) {
    // Network path: preserve the first two backslashes, normalize the rest
    const prefix = '\\\\';
    const rest = formatted.substring(2).replace(/\\+/g, '\\');
    return prefix + rest;
  } else {
    // Local path: reduce all consecutive backslashes to a single backslash
    return formatted.replace(/\\+/g, '\\');
  }
}

export function formatImagePath(p: string | null | undefined, size: string = 'w500'): string {
  if (!p) return "";

  if (p.startsWith('http://') || p.startsWith('https://')) {
    return p;
  }

  const isWindowsAbsolute = /^[A-Za-z]:[\\/]/.test(p) || p.startsWith('\\\\');
  const isUnixAbsolute = p.startsWith('/') && p.split('/').length > 2;

  if (isWindowsAbsolute || isUnixAbsolute) {
    return p;
  }

  const prefix = p.startsWith('/') ? '' : '/';
  return `https://image.tmdb.org/t/p/${size}${prefix}${p}`;
}

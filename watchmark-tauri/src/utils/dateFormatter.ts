export function formatLocaleDate(dateString: string): string {
  if (!dateString || dateString === '0000-00-00' || dateString === 'TBD') {
    return 'TBD';
  }

  // Create date considering timezone
  // Split the date string to reliably get year, month, day to construct Date in local timezone
  const parts = dateString.split('-');
  if (parts.length === 3) {
      const year = parseInt(parts[0], 10);
      const month = parseInt(parts[1], 10) - 1; // months are 0-indexed in JS
      const day = parseInt(parts[2], 10);
      const date = new Date(year, month, day);
      return new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', year: 'numeric' }).format(date);
  }

  return dateString;
}

export function formatRuntime(minutes: number): string {
  if (minutes < 60) return `${minutes}m`;
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return m === 0 ? `${h}h` : `${h}h ${m}m`;
}

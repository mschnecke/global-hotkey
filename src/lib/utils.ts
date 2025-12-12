/**
 * Utility functions for Global Hotkey application
 */

/**
 * Format a hotkey binding for display
 */
export function formatHotkey(modifiers: string[], key: string): string {
  const parts = [...modifiers, key];
  return parts.join(' + ');
}

/**
 * Generate a unique ID
 */
export function generateId(): string {
  return crypto.randomUUID();
}

/**
 * Get current ISO timestamp
 */
export function getTimestamp(): string {
  return new Date().toISOString();
}

/**
 * Debounce a function
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Truncate a string with ellipsis
 */
export function truncate(str: string, maxLength: number): string {
  if (str.length <= maxLength) return str;
  return str.slice(0, maxLength - 3) + '...';
}

/**
 * Extract filename from path
 */
export function getFilename(path: string): string {
  const separator = path.includes('\\') ? '\\' : '/';
  const parts = path.split(separator);
  return parts[parts.length - 1] || path;
}

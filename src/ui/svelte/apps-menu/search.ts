import type { StartMenuItem } from "@seelen-ui/lib/types";

/**
 * Normalizes a string for search by:
 * - Converting to lowercase
 * - Removing spaces, underscores, hyphens, parentheses, and other special characters
 */
function normalizeForSearch(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w_]/g, "");
}

/**
 * Searches for a query in a StartMenuItem
 * Matches against display_name, path, and umid
 */
export function matchesSearch(item: StartMenuItem, query: string): boolean {
  if (!query || query.trim().length === 0) {
    return true;
  }

  const normalizedQuery = normalizeForSearch(query);

  // Search in display name
  const normalizedDisplayName = normalizeForSearch(item.display_name);
  if (normalizedDisplayName.includes(normalizedQuery)) {
    return true;
  }

  // Search in path
  const normalizedPath = normalizeForSearch(item.path?.toString() || "");
  if (normalizedPath.includes(normalizedQuery)) {
    return true;
  }

  // Search in umid (app user model id)
  if (item.umid) {
    const normalizedUmid = normalizeForSearch(item.umid);
    if (normalizedUmid.includes(normalizedQuery)) {
      return true;
    }
  }

  return false;
}

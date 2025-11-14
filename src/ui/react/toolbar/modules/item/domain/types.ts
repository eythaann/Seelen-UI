/**
 * Domain types for the toolbar item module.
 * These types represent pure business logic without infrastructure dependencies.
 */

/**
 * Configuration options for the item scope.
 * Defines the data structure needed to create a scope for item evaluation.
 */
export interface ItemScopeOptions {
  itemId: string;
  extraVars?: Record<string, any>;
  fetchedData?: Record<string, any>;
}

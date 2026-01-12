import type { ZodSchema } from "zod";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { debounce } from "lodash";

export class PersistentRune<T> {
  private _value: T;
  private storeKey: string;
  private schema?: ZodSchema<T>;
  private debouncedSave: ReturnType<typeof debounce>;

  private constructor(storeKey: string, initial: T, schema?: ZodSchema<T>) {
    this.storeKey = storeKey;
    this.schema = schema;
    this._value = $state(initial);
    this.debouncedSave = debounce(this.saveToFile.bind(this), 500);
  }

  static async create<T>(
    storeKey: string,
    initial: T,
    schema?: ZodSchema<T>,
  ): Promise<PersistentRune<T>> {
    const rune = new PersistentRune(storeKey, initial, schema);
    const stored = await rune.loadFromFile();
    rune._value = stored ?? initial;
    return rune;
  }

  private async loadFromFile(): Promise<T | null> {
    try {
      const content = await invoke(SeelenCommand.ReadFile, {
        filename: `${this.storeKey}.json`,
      });

      const parsed = JSON.parse(content);
      if (this.schema) {
        return this.schema.parse(parsed);
      }

      return parsed as T;
    } catch {
      // File doesn't exist or is invalid, ignore
      return null;
    }
  }

  private async saveToFile(value: T): Promise<void> {
    try {
      await invoke(SeelenCommand.WriteFile, {
        filename: `${this.storeKey}.json`,
        content: JSON.stringify(value),
      });
    } catch (error) {
      console.error(`Failed to save to file (key: ${this.storeKey}):`, error);
    }
  }

  get value(): T {
    return this._value;
  }

  set value(value: T) {
    this._value = value;
    this.debouncedSave(value);
  }
}

/**
 * Helper function to create a PersistentRune
 *
 * @example
 * const count = await persistentRune('counter', 0);
 * count.value++; // Automatically saved to file
 *
 * @example with zod validation
 * import { z } from 'zod';
 * const schema = z.number();
 * const count = await persistentRune('counter', 0, schema);
 * count.value++; // Validated and saved to file
 */
export function persistentRune<T>(
  storeKey: string,
  initial: T,
  schema?: ZodSchema<T>,
): Promise<PersistentRune<T>> {
  return PersistentRune.create(storeKey, initial, schema);
}

import type { ZodSchema } from "zod";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { Signal } from "@preact/signals";
import { debounce } from "lodash";

export class PersistentSignal<T> extends Signal<T> {
  private _ready = false;
  private storeKey: string;
  private schema?: ZodSchema<T>;
  private debouncedSave: ReturnType<typeof debounce>;

  private constructor(storeKey: string, initial: T, schema?: ZodSchema<T>) {
    super(initial);
    this.storeKey = storeKey;
    this.schema = schema;
    this.debouncedSave = debounce(this.saveToFile.bind(this), 500);
  }

  static async create<T>(
    storeKey: string,
    initial: T,
    schema?: ZodSchema<T>,
  ): Promise<PersistentSignal<T>> {
    const s = new PersistentSignal(storeKey, initial, schema);
    const stored = await s.loadFromFile();
    if (stored !== null) s.value = stored; // _ready=false → no save triggered
    s._ready = true;
    return s;
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
    return super.value;
  }

  set value(v: T) {
    super.value = v;
    if (this._ready) {
      this.debouncedSave(v);
    }
  }
}

/**
 * Helper function to create a PersistentSignal
 *
 * @example
 * const count = await persistentSignal('counter', 0);
 * count.value++; // Automatically saved to file
 *
 * @example with zod validation
 * import { z } from 'zod';
 * const schema = z.number();
 * const count = await persistentSignal('counter', 0, schema);
 * count.value++; // Validated and saved to file
 */
export function persistentSignal<T, Z extends ZodSchema<T, any, any>>(
  storeKey: string,
  initial: T,
  schema?: Z,
): Promise<PersistentSignal<T>> {
  return PersistentSignal.create(storeKey, initial, schema);
}

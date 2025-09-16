/**
 * A generic, abstract class for managing an array-like collection of items.
 * @template T The type of elements stored in the list.
 */
export abstract class List<T = unknown> {
  /**
   * Constructor for the List class.
   * @param inner The internal array that stores the elements.
   * @throws Error if the provided array is not
   */
  constructor(protected inner: T[]) {
    if (!inner) {
      throw new Error('The inner array cannot be null or undefined.');
    }
    if (!Array.isArray(inner)) {
      throw new Error('The inner array must be an array.');
    }
  }

  public [Symbol.iterator](): Iterable<T> {
    return this.inner[Symbol.iterator]();
  }

  public get length(): number {
    return this.inner.length;
  }

  /**
   * Provides direct access to the internal array of items.
   * @returns A reference to the internal array of items.
   */
  public asArray(): T[] {
    return this.inner;
  }

  /**
   * Returns a copy of the internal array of items.
   * This ensures the internal array remains immutable when accessed via this method.
   * @returns A new array containing the elements of the internal array.
   */
  public all(): T[] {
    return [...this.inner];
  }
}

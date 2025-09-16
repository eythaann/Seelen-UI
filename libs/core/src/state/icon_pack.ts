import type {
  Icon as IIcon,
  IconPack,
  IconPack as IIconPack,
  SeelenCommandGetIconArgs,
  UniqueIconPackEntry,
} from "@seelen-ui/types";
import { List } from "../utils/List.ts";
import { newFromInvoke, newOnEvent } from "../utils/State.ts";
import { invoke, SeelenCommand, SeelenEvent, type UnSubscriber } from "../handlers/mod.ts";
import { Settings } from "./settings/mod.ts";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";

export class IconPackList extends List<IIconPack> {
  static getAsync(): Promise<IconPackList> {
    return newFromInvoke(this, SeelenCommand.StateGetIconPacks);
  }

  static onChange(cb: (payload: IconPackList) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.StateIconPacksChanged);
  }
}

/**
 * Class helper to allow easy use of icon packs
 */
export class IconPackManager {
  private callbacks: Set<() => void> = new Set();
  private unlisteners: UnlistenFn[] = [];
  private isListeningForChanges = false;

  /// list of active icon packs and fully resolved paths
  private activeIconPacks: IconPack[] = [];

  protected constructor(
    protected _availableIconPacks: IconPack[],
    protected _activeIconPackIds: string[],
  ) {}

  get iconPacks(): IconPack[] {
    return this._availableIconPacks;
  }

  get activeIconPackIds(): string[] {
    return this._activeIconPackIds;
  }

  protected resolveAvailableIcons(): void {
    for (const pack of this._availableIconPacks) {
      const path = `${pack.metadata.path}`;

      if (pack.missing) {
        pack.missing = resolveIcon(path, pack.missing);
      }

      for (const entry of pack.entries) {
        if (entry.type === "unique") {
          entry.path = entry.path?.toLowerCase() || null;
        }

        if (entry.type === "shared") {
          entry.extension = entry.extension.toLowerCase();
        }

        if (entry.icon) {
          entry.icon = resolveIcon(path, entry.icon);
        }
      }
    }
  }

  protected cacheActiveIconPacks(): void {
    this.activeIconPacks = [];
    for (const key of this._activeIconPackIds.toReversed()) {
      const pack = this._availableIconPacks.find((p) => p.id === key);
      if (pack) {
        this.activeIconPacks.push(pack);
      }
    }
  }

  /**
   * Creates an instance of IconPackManager. This intance will be updated when
   * the list of icon packs or the settings changes, so just having one global instance is enough.
   *
   * @returns A new instance of IconPackManager
   */
  public static async create(): Promise<IconPackManager> {
    const instance = new IconPackManager(
      (await IconPackList.getAsync()).asArray(),
      (await Settings.getAsync()).inner.activeIconPacks,
    );
    instance.resolveAvailableIcons();
    instance.cacheActiveIconPacks();
    return instance;
  }

  /**
   * Registers a callback to be invoked when the list of active icon packs changes.
   * This method also sets up listeners to detect changes in the icon pack list and
   * the active icon packs settings. If no callbacks are registered beforehand, the
   * listeners are initialized. When no callbacks remain registered, the listeners are stopped.
   *
   * @param {() => void} cb - The callback to be invoked when the list of active icon packs changes.
   *                          This callback takes no arguments and returns no value.
   * @returns {Promise<UnlistenFn>} A promise that resolves to an `UnlistenFn` function. When invoked,
   *                                this function unregisters the callback and stops listening for changes
   *                                if no other callbacks are registered.
   *
   * @example
   * const manager = await IconPackManager.create();
   * const unlisten = await manager.onChange(() => {
   *   console.log("Icon packs changed: ", manager.actives);
   * });
   *
   * // Later, to stop listening for changes:
   * unlisten();
   *
   * @remarks
   * - The `this` context inside the callback refers to the `IconPackManager` instance, provided the callback
   *   is not rebound to another context (e.g., using `bind`, `call`, or `apply`).
   * - If the callback is defined as an arrow function, `this` will be lexically bound to the surrounding context.
   * - If the callback is a regular function, ensure it is bound correctly to avoid `this` being `undefined` (in strict mode)
   *   or the global object (in non-strict mode).
   *
   * @see {@link IconPackManager} for the class this method belongs to.
   * @see {@link UnlistenFn} for the type of the function returned by this method.
   */
  public async onChange(cb: () => void): Promise<UnlistenFn> {
    this.callbacks.add(cb);

    if (!this.isListeningForChanges) {
      this.isListeningForChanges = true;
      const unlistenerIcons = await IconPackList.onChange((list) => {
        this._availableIconPacks = JSON.parse(JSON.stringify(list.all()));
        this.resolveAvailableIcons();
        this.cacheActiveIconPacks();
        this.callbacks.forEach((cb) => cb());
      });
      const unlistenerSettings = await Settings.onChange((settings) => {
        this._activeIconPackIds = settings.inner.activeIconPacks;
        this.cacheActiveIconPacks();
        this.callbacks.forEach((cb) => cb());
      });
      this.unlisteners = [unlistenerIcons, unlistenerSettings];
    }

    return () => {
      this.callbacks.delete(cb);
      if (this.callbacks.size === 0) {
        this.unlisteners.forEach((unlisten) => unlisten());
        this.unlisteners = [];
        this.isListeningForChanges = false;
      }
    };
  }

  /**
   * Returns the icon path for an app or file. If no icon is available, returns `null`.
   *
   * The search for icons follows this priority order:
   * 1. UMID (App User Model Id)
   * 2. Full path
   * 3. Filename (this is only used to match executable files like .exe)
   * 4. Extension
   *
   * @param {Object} args - Arguments for retrieving the icon path.
   * @param {string} [args.path] - The full path to the app or file.
   * @param {string} [args.umid] - The UMID of the app.
   * @returns {string | null} - The path to the icon, or `null` if no icon is found.
   *
   * @example
   * // Example 1: Get icon by full path
   * const iconPath = instance.getIconPath({
   *   path: "C:\\Program Files\\Steam\\steam.exe"
   * });
   *
   * // Example 2: Get icon by UMID
   * const iconPath = instance.getIconPath({
   *   umid: "Seelen.SeelenUI_p6yyn03m1894e!App"
   * });
   */
  public getIconPath(args: SeelenCommandGetIconArgs): IIcon | null {
    const { path, umid, __seen = new Set<string>() } = args as
      & SeelenCommandGetIconArgs
      & { __seen?: Set<string> };
    // If neither path nor UMID is provided, return null
    if (!path && !umid) {
      return null;
    }

    const lowerPath = path?.toLowerCase();
    const extension = lowerPath?.split(".").pop();

    for (const pack of this.activeIconPacks) {
      let entry: UniqueIconPackEntry | undefined;

      if (umid) {
        entry = pack.entries.find((e) => e.type === "unique" && !!e.umid && e.umid === umid) as UniqueIconPackEntry;
      }

      if (!entry && lowerPath) {
        entry = pack.entries.find((e) => {
          if (e.type !== "unique" || !e.path) {
            return false;
          }

          if (e.path === lowerPath) {
            return true;
          }

          // only search for filename in case of executable files
          if (extension === "exe") {
            const filename = lowerPath.split("\\").pop();
            if (filename && e.path.endsWith(filename)) {
              return true;
            }
          }
          return false;
        }) as UniqueIconPackEntry;
      }

      if (entry) {
        if (entry.redirect) {
          // break circular references
          if (__seen.has(entry.redirect)) {
            return null;
          }
          __seen.add(entry.redirect);
          return this.getIconPath(
            { path: entry.redirect, __seen } as SeelenCommandGetIconArgs,
          );
        }

        if (entry.icon) {
          return entry.icon;
        }
      }
    }

    // search by file extension
    if (!extension) {
      return null;
    }

    for (const pack of this.activeIconPacks) {
      const icon = pack.entries.find((e) => {
        return e.type === "shared" && e.extension === extension;
      });

      if (icon) {
        return icon.icon;
      }
    }

    // If no icon is found in any icon pack, return null
    return null;
  }

  public getIcon({ path, umid }: SeelenCommandGetIconArgs): IIcon | null {
    const iconPath = this.getIconPath({ path, umid });
    return iconPath ? resolveAsSrc(iconPath) : null;
  }

  /**
   * Will return the special missing icon path from the highest priority icon pack.
   * If no icon pack haves a missing icon, will return null.
   */
  public getMissingIconPath(): IIcon | null {
    for (const pack of this.activeIconPacks) {
      if (pack.missing) {
        return pack.missing;
      }
    }
    return null;
  }

  /**
   * Will return the special missing icon SRC from the highest priority icon pack.
   * If no icon pack haves a missing icon, will return null.
   */
  public getMissingIcon(): IIcon | null {
    const iconPath = this.getMissingIconPath();
    return iconPath ? resolveAsSrc(iconPath) : null;
  }

  /**
   * Will return the specific icon path from the highest priority icon pack.
   * If no icon pack haves the searched icon, will return null.
   */
  public getCustomIconPath(name: string): IIcon | null {
    for (const pack of this.activeIconPacks) {
      const entry = pack.entries.find((e) => e.type === "custom" && e.key === name);
      if (entry) {
        return entry.icon;
      }
    }
    return null;
  }

  /**
   * Will return the specific icon SRC from the highest priority icon pack.
   * If no icon pack haves the searched icon, will return null.
   */
  public getCustomIcon(name: string): IIcon | null {
    const iconPath = this.getCustomIconPath(name);
    return iconPath ? resolveAsSrc(iconPath) : null;
  }

  /**
   * This method doesn't take in care icon packs, just extracts the inherited icon into system's icon pack
   * if it's not already there.
   *
   * @param filePath The path to the app could be umid o full path
   * @example
   * const iconPath = instance.extractIcon({
   *   path: "C:\\Program Files\\Steam\\steam.exe"
   * });
   * const iconPath = instance.extractIcon({
   *   umid: "Seelen.SeelenUI_p6yyn03m1894e!App"
   * });
   */
  public static requestIconExtraction(
    obj: SeelenCommandGetIconArgs,
  ): Promise<void> {
    return invoke(SeelenCommand.GetIcon, obj);
  }

  /**
   * This will delete all stored icons on the system icon pack.\
   * All icons should be regenerated after calling this method.
   */
  public static clearCachedIcons(): Promise<void> {
    return invoke(SeelenCommand.StateDeleteCachedIcons);
  }
}

function resolveIcon(parent: string, icon: IIcon): IIcon {
  return {
    base: icon.base ? `${parent}\\${icon.base}` : null,
    light: icon.light ? `${parent}\\${icon.light}` : null,
    dark: icon.dark ? `${parent}\\${icon.dark}` : null,
    mask: icon.mask ? `${parent}\\${icon.mask}` : null,
    isAproximatelySquare: icon.isAproximatelySquare,
  };
}

function resolveAsSrc(icon: IIcon): IIcon {
  return {
    base: icon.base ? convertFileSrc(icon.base) : null,
    light: icon.light ? convertFileSrc(icon.light) : null,
    dark: icon.dark ? convertFileSrc(icon.dark) : null,
    mask: icon.mask ? convertFileSrc(icon.mask) : null,
    isAproximatelySquare: icon.isAproximatelySquare,
  };
}

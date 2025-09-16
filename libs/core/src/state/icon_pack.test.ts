import type { Icon, IconPack, IconPackId, ResourceMetadata } from "@seelen-ui/types";
import { IconPackManager } from "./icon_pack.ts";
import { assertEquals } from "@std/assert";

// Custom assertion for null values
function assertNull(value: unknown): void {
  return assertEquals(value, null);
}

function onlyBase(x: string): Icon {
  return {
    base: x,
    light: null,
    dark: null,
    mask: null,
    isAproximatelySquare: false,
  };
}

// Constants for test icons
const GOT_BY_PATH = "GOT_BY_PATH";
const GOT_BY_FILENAME = "GOT_BY_FILENAME";
const GOT_BY_EXTENSION = "GOT_BY_EXTENSION";
const GOT_BY_UMID = "GOT_BY_UMID";

const A_PATH = "path\\to\\a";
const B_PATH = "path\\to\\b";
const C_PATH = "path\\to\\c";

// Deep clone helper to ensure test isolation
const cloneIconPack = (pack: IconPack): IconPack => JSON.parse(JSON.stringify(pack));

// Factory function for mock icon packs
const createMockIconPacks = (): {
  packA: IconPack;
  packB: IconPack;
  packC: IconPack;
} => ({
  packA: {
    id: "a" as IconPackId,
    metadata: { path: A_PATH } as ResourceMetadata,
    missing: onlyBase("MissingIconA.png"),
    entries: [
      // Unique entries (apps)
      {
        type: "unique",
        umid: "MSEdge",
        redirect: null,
        path: "C:\\Program Files (x86)\\Microsoft\\Edge\\msedge.exe",
        icon: onlyBase(GOT_BY_UMID),
      },
      {
        type: "unique",
        umid: null,
        redirect: null,
        path: "C:\\Windows\\explorer.exe",
        icon: onlyBase(GOT_BY_PATH),
      },
      {
        type: "unique",
        umid: null,
        redirect: null,
        path: "C:\\Program Files (x86)\\Some\\App\\filenameApp.exe",
        icon: onlyBase(GOT_BY_PATH),
      },
      // Shared entries (file extensions)
      {
        type: "shared",
        extension: "txt",
        icon: onlyBase(GOT_BY_EXTENSION),
      },
      {
        type: "shared",
        extension: "png",
        icon: onlyBase(GOT_BY_EXTENSION),
      },
      {
        type: "shared",
        extension: "jpg",
        icon: onlyBase(GOT_BY_EXTENSION),
      },
      // Custom entries
      {
        type: "custom",
        key: "my-custom-icon",
        icon: onlyBase("CustomA.png"),
      },
    ],
    remoteEntries: [],
    downloaded: false,
  },
  packB: {
    id: "b" as IconPackId,
    metadata: { path: B_PATH } as ResourceMetadata,
    missing: onlyBase("MissingIconB.png"),
    entries: [
      // Unique entries (apps)
      {
        type: "unique",
        umid: null,
        redirect: null,
        path: "C:\\Windows\\explorer.exe",
        icon: onlyBase(GOT_BY_PATH),
      },
      {
        type: "unique",
        umid: null,
        redirect: null,
        path: "filenameApp.exe",
        icon: onlyBase(GOT_BY_FILENAME),
      },
      // Shared entries (file extensions)
      {
        type: "shared",
        extension: "txt",
        icon: onlyBase(GOT_BY_EXTENSION),
      },
      // Custom entries
      {
        type: "custom",
        key: "my-custom-icon",
        icon: onlyBase("CustomB.png"),
      },
    ],
    remoteEntries: [],
    downloaded: false,
  },
  packC: {
    id: "c" as IconPackId,
    metadata: { path: C_PATH } as ResourceMetadata,
    missing: null,
    entries: [
      // Unique entries (apps)
      {
        type: "unique",
        umid: null,
        path: "C:\\folder\\app1.exe",
        icon: onlyBase(GOT_BY_PATH),
        redirect: null,
      },
    ],
    remoteEntries: [],
    downloaded: false,
  },
});

// Test context manager for cleaner test setup
class IconPackManagerTestContext {
  private manager: IconPackManagerMock;

  // Default active packs: ['b', 'a'] (note 'a' has higher priority as it's last)
  constructor(initialActives: string[] = ["b", "a"]) {
    const mocks = createMockIconPacks();
    this.manager = new IconPackManagerMock(
      [
        cloneIconPack(mocks.packA),
        cloneIconPack(mocks.packB),
        cloneIconPack(mocks.packC),
      ],
      initialActives,
    );
  }

  get instance(): IconPackManagerMock {
    return this.manager;
  }

  // Fluent interface for changing active packs
  withActives(actives: string[]): this {
    this.manager.setActives(actives);
    return this;
  }
}

// Mock implementation of IconPackManager for testing
class IconPackManagerMock extends IconPackManager {
  constructor(packs: IconPack[], activeKeys: string[]) {
    super(packs, activeKeys);
    this.resolveAvailableIcons();
    this.cacheActiveIconPacks();
  }

  public setActives(actives: string[]): void {
    this._activeIconPackIds = actives;
    this.cacheActiveIconPacks();
  }
}

Deno.test("IconPackManager", async (t) => {
  await t.step("Icon lookup functionality", async (t) => {
    await t.step("should return null for non-existent paths or UMIDs", () => {
      const ctx = new IconPackManagerTestContext();
      // Non-existent path
      assertNull(
        ctx.instance.getIconPath({ path: "C:\\nonexistent\\path.exe" }),
      );
      // Non-existent UMID
      assertNull(ctx.instance.getIconPath({ umid: "NonexistentUMID" }));
    });

    await t.step("should ignore inactive icon packs", () => {
      // Only 'a' and 'b' are active by default
      const ctx = new IconPackManagerTestContext();
      // This path only exists in packC which is inactive
      assertNull(ctx.instance.getIconPath({ path: "C:\\folder\\app1.exe" }));
    });

    await t.step(
      "should respect cascading priority order (last has highest priority)",
      () => {
        // Default order is ['b', 'a'] so 'a' has higher priority
        const ctx = new IconPackManagerTestContext(["b", "a"]);

        // 'a' should take priority for explorer.exe (last in active list)
        assertEquals(
          ctx.instance.getIconPath({ path: "C:\\Windows\\explorer.exe" }),
          onlyBase(`${A_PATH}\\${GOT_BY_PATH}`),
        );

        // After changing priority to ['a', 'b'], 'b' should now have priority
        assertEquals(
          ctx.withActives(["a", "b"]).instance.getIconPath({
            path: "C:\\Windows\\explorer.exe",
          }),
          onlyBase(`${B_PATH}\\${GOT_BY_PATH}`),
        );
      },
    );

    await t.step("should prioritize UMID over path matching", () => {
      const ctx = new IconPackManagerTestContext(["b", "a"]); // 'a' has priority
      // Should match UMID in packA even though path exists in both packs
      assertEquals(
        ctx.instance.getIconPath({
          path: "C:\\Program Files (x86)\\Microsoft\\Edge\\msedge.exe",
          umid: "MSEdge",
        }),
        onlyBase(`${A_PATH}\\${GOT_BY_UMID}`),
      );
    });

    await t.step(
      "should match by filename when higher priority pack has filename match",
      () => {
        // With ['b', 'a'] order, packB has filename match that should take priority
        const ctx = new IconPackManagerTestContext(["a", "b"]);
        assertEquals(
          ctx.instance.getIconPath({
            path: "C:\\Program Files (x86)\\Some\\App\\filenameApp.exe",
          }),
          onlyBase(`${B_PATH}\\${GOT_BY_FILENAME}`),
        );
      },
    );

    await t.step("should match files by extension with priority order", () => {
      const ctx = new IconPackManagerTestContext(["b", "a"]); // 'a' has priority

      // .txt exists in both packs - should use 'a' (higher priority)
      assertEquals(
        ctx.instance.getIconPath({ path: "C:\\Some\\App\\someFile.txt" }),
        onlyBase(`${A_PATH}\\${GOT_BY_EXTENSION}`),
      );

      // .png only exists in packA
      assertEquals(
        ctx.instance.getIconPath({ path: "C:\\Some\\App\\someFile.png" }),
        onlyBase(`${A_PATH}\\${GOT_BY_EXTENSION}`),
      );

      // When we change priority to ['a', 'b'], 'b' should have priority for .txt
      assertEquals(
        ctx.withActives(["a", "b"]).instance.getIconPath({
          path: "C:\\Some\\App\\someFile.txt",
        }),
        onlyBase(`${B_PATH}\\${GOT_BY_EXTENSION}`),
      );
    });
  });

  await t.step("Missing icon functionality", async (t) => {
    await t.step(
      "should return missing icon from highest priority pack (last in active list)",
      () => {
        // With ['b', 'a'], 'a' has priority
        const ctx = new IconPackManagerTestContext(["b", "a"]);
        assertEquals(
          ctx.instance.getMissingIconPath(),
          onlyBase(`${A_PATH}\\MissingIconA.png`),
        );
      },
    );

    await t.step(
      "should fallback when higher priority pack has no missing icon",
      () => {
        // packC has no missing icon, should fallback to packB
        const ctx = new IconPackManagerTestContext(["c", "b"]);
        assertEquals(
          ctx.instance.getMissingIconPath(),
          onlyBase(`${B_PATH}\\MissingIconB.png`),
        );
      },
    );

    await t.step(
      "should return null when no active packs have missing icons",
      () => {
        const ctx = new IconPackManagerTestContext(["c"]); // packC has no missing icon
        assertNull(ctx.instance.getMissingIconPath());
      },
    );
  });

  await t.step("Custom icon functionality", async (t) => {
    await t.step("should return custom icon from highest priority pack", () => {
      // With ['b', 'a'], 'a' has priority
      const ctx = new IconPackManagerTestContext(["b", "a"]);
      assertEquals(
        ctx.instance.getCustomIconPath("my-custom-icon"),
        onlyBase(`${A_PATH}\\CustomA.png`),
      );
    });

    await t.step(
      "should fallback when custom icon not found in higher priority pack",
      () => {
        // packC has no custom icons, should fallback to packA
        const ctx = new IconPackManagerTestContext(["c", "a"]);
        assertEquals(
          ctx.instance.getCustomIconPath("my-custom-icon"),
          onlyBase(`${A_PATH}\\CustomA.png`),
        );
      },
    );

    await t.step(
      "should return null when custom icon not found in any active pack",
      () => {
        const ctx = new IconPackManagerTestContext(["c"]);
        assertNull(ctx.instance.getCustomIconPath("non-existent-icon"));
      },
    );
  });

  await t.step("Redirect functionality", async (t) => {
    await t.step(
      "should follow redirect and ignore icon when redirect is present",
      () => {
        const ctx = new IconPackManagerTestContext(["a", "b"]);
        // Add entry with both redirect and icon (icon should be ignored)
        ctx.instance.iconPacks[0].entries.push({
          type: "unique",
          umid: "RedirectTest",
          path: "C:\\redirect\\source.exe",
          redirect: "C:\\redirect\\target.exe",
          icon: onlyBase(A_PATH + "\\ThisShouldBeIgnored.png"),
        });
        // Add target entry to packB
        ctx.instance.iconPacks[1].entries.push({
          type: "unique",
          umid: null,
          path: "C:\\redirect\\target.exe",
          redirect: null,
          icon: onlyBase(B_PATH + "\\RedirectTargetIcon.png"),
        });

        assertEquals(
          ctx.instance.getIconPath({ umid: "RedirectTest" }),
          onlyBase(B_PATH + "\\RedirectTargetIcon.png"),
        );
      },
    );

    await t.step(
      "should not use icon when redirect points to non-existent path",
      () => {
        const ctx = new IconPackManagerTestContext(["a"]);
        // Add entry with redirect to non-existent path and icon
        ctx.instance.iconPacks[0].entries.push({
          type: "unique",
          umid: "BadRedirect",
          path: "C:\\redirect\\source.exe",
          redirect: "C:\\nonexistent\\path.exe",
          icon: onlyBase(A_PATH + "\\ShouldNotUseThis.png"), // <-- should be ignored inclusively if redirect points to non-existent path
        });

        assertNull(ctx.instance.getIconPath({ umid: "BadRedirect" }));
      },
    );

    await t.step(
      "should follow redirect chain until icon is found or no more redirects",
      () => {
        const ctx = new IconPackManagerTestContext(["a", "b", "c"]);
        // First redirect (icon should be ignored)
        ctx.instance.iconPacks[0].entries.push({
          type: "unique",
          umid: "ChainRedirect",
          path: "C:\\redirect\\start.exe",
          redirect: "C:\\redirect\\middle.exe",
          icon: onlyBase(A_PATH + "\\IgnoreThis.png"),
        });
        // Second redirect (no icon)
        ctx.instance.iconPacks[1].entries.push({
          type: "unique",
          umid: null,
          path: "C:\\redirect\\middle.exe",
          redirect: "C:\\redirect\\final.exe",
          icon: null,
        });
        // Final target with icon
        ctx.instance.iconPacks[2].entries.push({
          type: "unique",
          umid: null,
          path: "C:\\redirect\\final.exe",
          redirect: null,
          icon: onlyBase(C_PATH + "\\FinalIcon.png"),
        });

        assertEquals(
          ctx.instance.getIconPath({ umid: "ChainRedirect" }),
          onlyBase(C_PATH + "\\FinalIcon.png"),
        );
      },
    );

    await t.step(
      "should return null if redirect chain ends without finding an icon",
      () => {
        const ctx = new IconPackManagerTestContext(["a", "b"]);
        // First redirect
        ctx.instance.iconPacks[0].entries.push({
          type: "unique",
          umid: "BrokenChain",
          path: "C:\\redirect\\start.exe",
          redirect: "C:\\redirect\\missing.exe",
          icon: onlyBase(A_PATH + "\\IgnoreMe.png"),
        });

        // No matching entry for the redirect target
        assertNull(ctx.instance.getIconPath({ umid: "BrokenChain" }));
      },
    );

    await t.step("should handle redirect to extension match", () => {
      const ctx = new IconPackManagerTestContext(["a", "b"]);
      // Redirect to a file with specific extension
      ctx.instance.iconPacks[0].entries.push({
        type: "unique",
        umid: "RedirectToExtension",
        path: "C:\\some\\app.exe",
        redirect: "C:\\some\\file.txt", // <-- redirect to txt file
        icon: onlyBase(A_PATH + "\\WrongIcon.png"), // <-- should be ignored
      });

      assertEquals(
        ctx.instance.getIconPath({ umid: "RedirectToExtension" }),
        onlyBase(B_PATH + "\\GOT_BY_EXTENSION"),
      );
    });
  });

  await t.step("should handle circular references and return null", () => {
    const ctx = new IconPackManagerTestContext(["a", "b"]);

    ctx.instance.iconPacks[0].entries.push({
      type: "unique",
      umid: "CircularRef1",
      path: "C:\\circle\\app1.exe",
      redirect: "C:\\circle\\app2.exe",
      icon: onlyBase("IgnoredIcon1.png"),
    });

    ctx.instance.iconPacks[1].entries.push({
      type: "unique",
      umid: "CircularRef2",
      path: "C:\\circle\\app2.exe",
      redirect: "C:\\circle\\app1.exe",
      icon: onlyBase("IgnoredIcon2.png"),
    });

    assertNull(ctx.instance.getIconPath({ umid: "CircularRef1" }));
    assertNull(ctx.instance.getIconPath({ path: "C:\\circle\\app1.exe" }));
    assertNull(ctx.instance.getIconPath({ path: "C:\\circle\\app2.exe" }));
  });

  await t.step("should detect self-references and return null", () => {
    const ctx = new IconPackManagerTestContext(["a"]);

    // Configurar autorreferencia
    ctx.instance.iconPacks[0].entries.push({
      type: "unique",
      umid: "SelfRef",
      path: "C:\\circle\\self.exe",
      redirect: "C:\\circle\\self.exe", // Se redirige a sí mismo
      icon: onlyBase("IgnoredIcon.png"),
    });

    assertNull(ctx.instance.getIconPath({ umid: "SelfRef" }));
  });

  await t.step(
    "should detect longer circular references and return null",
    () => {
      const ctx = new IconPackManagerTestContext(["a", "b", "c"]);

      // Configurar referencia circular más larga (A -> B -> C -> A)
      ctx.instance.iconPacks[0].entries.push({
        type: "unique",
        umid: null,
        path: "C:\\circle\\a.exe",
        redirect: "C:\\circle\\b.exe",
        icon: onlyBase("IgnoredA.png"),
      });

      ctx.instance.iconPacks[1].entries.push({
        type: "unique",
        umid: null,
        path: "C:\\circle\\b.exe",
        redirect: "C:\\circle\\c.exe",
        icon: onlyBase("IgnoredB.png"),
      });

      ctx.instance.iconPacks[2].entries.push({
        type: "unique",
        umid: null,
        path: "C:\\circle\\c.exe",
        redirect: "C:\\circle\\a.exe",
        icon: onlyBase("IgnoredC.png"),
      });

      assertNull(ctx.instance.getIconPath({ path: "C:\\circle\\a.exe" }));
    },
  );
});

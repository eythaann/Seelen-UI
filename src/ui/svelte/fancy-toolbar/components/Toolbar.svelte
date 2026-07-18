<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import type { ContextMenu, ContextMenuItem, PluginId, WidgetId } from "@seelen-ui/lib/types";
  import { Alignment, FancyToolbarSide } from "@seelen-ui/lib/types";
  import { DragDropProvider, DragOverlay } from "@dnd-kit/svelte";
  import { move } from "@dnd-kit/helpers";
  import { onDestroy, onMount } from "svelte";
  import { BackgroundByLayers } from "libs/ui/svelte/components/BackgroundByLayers";
  import { getResourceText } from "libs/ui/react/utils/index.ts";
  import { DND_PLUGINS, DND_SENSORS } from "libs/ui/dnd.ts";
  import { t, locale } from "../i18n/index.ts";
  import {
    toolbarState,
    toolbarActions,
    plugins,
    restoreStateToDefault,
    listToGroups,
  } from "../state/items.svelte.ts";
  import { settingsState } from "../state/settings.svelte.ts";
  import { hiddenByAutohide, setToolbarIsDraggingItem } from "../state/hidden.svelte.ts";
  import { windowsState } from "../state/windows.svelte.ts";
  import { matchIds } from "../utils.ts";
  import ItemsGroup from "./ItemsGroup.svelte";
  import CornerAction from "./CornerAction.svelte";
  import Item from "./Item.svelte";

  // ── Derived splits ───────────────────────────────────────────────────────

  const groups = $derived(listToGroups(toolbarState.items));

  // dnd-kit's `move()` reorders by index within the full, unfiltered items
  // array, so sortables must report their true index there, not their
  // position within the (possibly filtered) rendered group.
  const itemIndexById = $derived.by(() => {
    const map = new Map<string, number>();
    toolbarState.items.forEach((item, i) => {
      map.set(typeof item === "string" ? item : item.id, i);
    });
    return map;
  });

  // ── Context menu ─────────────────────────────────────────────────────────

  const identifier = crypto.randomUUID();
  const modulesIdentifier = crypto.randomUUID();
  const onContextMenuClickEvent = "onContextMenuClick";
  const onTogglePluginEvent = "onTogglePlugin";

  let unlistenContextMenuClick: (() => void) | undefined;
  let unlistenTogglePlugin: (() => void) | undefined;

  onMount(() => {
    Widget.self.webview
      .listen(onContextMenuClickEvent, ({ payload }) => {
        const { key } = payload as any;
        if (key === "reoder") {
          toolbarState.state = {
            ...toolbarState.state,
            isReorderDisabled: !toolbarState.isReorderDisabled,
          };
        }
        if (key === "task_manager") {
          invoke(SeelenCommand.OpenFile, { path: "Taskmgr.exe" });
        }
        if (key === "settings") {
          invoke(SeelenCommand.TriggerWidget, {
            payload: { id: "@seelen/settings" as WidgetId },
          });
        }
        if (key === "restore") {
          restoreStateToDefault();
        }
      })
      .then((fn) => {
        unlistenContextMenuClick = fn;
      });

    Widget.self.webview
      .listen(onTogglePluginEvent, ({ payload }) => {
        const { key: pluginId, checked } = payload as any;
        if (checked) {
          toolbarActions.addItem(pluginId);
        } else {
          toolbarActions.removeItem(pluginId);
        }
      })
      .then((fn) => {
        unlistenTogglePlugin = fn;
      });
  });

  onDestroy(() => {
    unlistenContextMenuClick?.();
    unlistenTogglePlugin?.();
  });

  function buildContextMenu(): ContextMenu {
    const language = locale.value;

    function isAlreadyAdded(id: PluginId): boolean {
      return toolbarState.items.some((item) => item === id);
    }

    return {
      identifier,
      items: [
        {
          type: "Submenu",
          icon: "CgExtensionAdd",
          label: $t("context_menu.modules"),
          identifier: modulesIdentifier,
          items: [
            {
              type: "Item",
              key: "restore",
              icon: "TbRestore",
              label: $t("context_menu.restore"),
              callbackEvent: onContextMenuClickEvent,
            },
            { type: "Separator" },
            ...plugins.value
              .map<Extract<ContextMenuItem, { type: "Item" }>>((plugin) => ({
                type: "Item",
                key: plugin.id,
                label: getResourceText(plugin.metadata.displayName, language),
                icon: plugin.icon,
                callbackEvent: onTogglePluginEvent,
                checked: isAlreadyAdded(plugin.id),
              }))
              .toSorted((p1, p2) => p1.label.localeCompare(p2.label)),
          ],
        },
        { type: "Separator" },
        {
          type: "Item",
          key: "reoder",
          icon: toolbarState.isReorderDisabled ? "VscUnlock" : "VscLock",
          label: $t(
            toolbarState.isReorderDisabled
              ? "context_menu.reorder_enable"
              : "context_menu.reorder_disable",
          ),
          callbackEvent: onContextMenuClickEvent,
        },
        {
          type: "Item",
          key: "task_manager",
          icon: "PiChartLineFill",
          label: $t("context_menu.task_manager"),
          callbackEvent: onContextMenuClickEvent,
          checked: null,
          disabled: false,
        },
        {
          type: "Item",
          key: "settings",
          icon: "RiSettings4Fill",
          label: $t("context_menu.settings"),
          callbackEvent: onContextMenuClickEvent,
        },
      ],
    };
  }

  function handleContextMenu() {
    const alignY =
      settingsState.position === FancyToolbarSide.Bottom ? Alignment.End : Alignment.Start;
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...buildContextMenu(), alignX: Alignment.Center, alignY },
      forwardTo: null,
    });
  }

  // ── DnD ──────────────────────────────────────────────────────────────────

  function handleDragOver(event: any) {
    const temp = toolbarState.items.map((item) => (typeof item === "string" ? item : item.id));
    const newIds = move(temp, event);
    toolbarState.items = newIds.map((id) => toolbarState.items.find((i) => matchIds(i, id))!);
  }

  function handleDragStart() {
    setToolbarIsDraggingItem(true);
  }

  function handleDragEnd() {
    setToolbarIsDraggingItem(false);
  }

  // ── Resolve overlay item ──────────────────────────────────────────────────

  function resolveOverlayModule(sourceId: string) {
    const entry = toolbarState.items.find((i) => matchIds(i, sourceId));
    if (!entry) return null;
    if (typeof entry === "string") {
      const plugin = plugins.value.find((p) => p.id === entry);
      if (!plugin) return null;
      return { ...(plugin.plugin as any), id: entry };
    }
    return entry;
  }
</script>

<div
  role="toolbar"
  tabindex="0"
  class="ft-bar {settingsState.position.toLowerCase()}"
  class:ft-bar-hidden={hiddenByAutohide.value}
  data-has-margin={!!settingsState.margin}
  data-there-is-maximized-on-background={windowsState.thereIsMaximizedOnBg}
  oncontextmenu={handleContextMenu}
>
  <CornerAction />
  <BackgroundByLayers />

  <DragDropProvider
    plugins={DND_PLUGINS}
    sensors={DND_SENSORS}
    onDragStart={handleDragStart}
    onDragOver={handleDragOver}
    onDragEnd={handleDragEnd}
  >
    <ItemsGroup id="left" items={groups.left} {itemIndexById} />
    <ItemsGroup id="center" items={groups.center} {itemIndexById} />
    <ItemsGroup id="right" items={groups.right} {itemIndexById} />

    <DragOverlay>
      {#snippet children(source)}
        {@const module = resolveOverlayModule(source.id as string)}
        {#if module}
          <Item {module} />
        {/if}
      {/snippet}
    </DragOverlay>
  </DragDropProvider>

  <CornerAction />
</div>

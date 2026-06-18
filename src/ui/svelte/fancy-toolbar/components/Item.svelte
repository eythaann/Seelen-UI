<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import {
    Alignment,
    FancyToolbarSide,
    type ContextMenu,
    type ToolbarItem,
    type WidgetId,
  } from "@seelen-ui/lib/types";
  import type { createSortable } from "@dnd-kit/svelte/sortable";
  import { t } from "../i18n/index.ts";
  import { EvaluateAction } from "../actionEvaluator.ts";
  import { evalSandboxedCode, stringFromEvaluated } from "../evaluatedComponents.ts";
  import { buildItemScope } from "../state/scopes.svelte.ts";
  import { toolbarActions } from "../state/items.svelte.ts";
  import { settingsState } from "../state/settings.svelte.ts";
  import { triggerWidget } from "../widgetTrigger.ts";
  import { styleToString } from "../utils.ts";
  import EvaluatedComponents from "../EvaluatedComponents.svelte";

  interface Props {
    module: ToolbarItem;
    sortable?: ReturnType<typeof createSortable> | null;
  }

  let { module: self, sortable = null }: Props = $props();

  const noopAttach = () => {};

  // ── Remote data fetching ─────────────────────────────────────────────────

  let fetchedData = $state<Record<string, any>>({});

  $effect(() => {
    const remoteData = self.remoteData ?? {};
    const intervals: Record<string, ReturnType<typeof setInterval>> = {};
    let mounted = true;

    async function fetchKey(key: string, rd: any) {
      if (!mounted) return;
      try {
        const response = await fetch(rd.url, rd.requestInit as RequestInit);
        const data = response.headers.get("Content-Type")?.includes("application/json")
          ? await response.json()
          : await response.text();
        if (mounted) {
          fetchedData = { ...fetchedData, [key]: data };
        }
      } catch (err) {
        console.error(`Error fetching ${key}:`, err);
      }
    }

    for (const [key, rd] of Object.entries(remoteData)) {
      if (!rd) continue;
      fetchKey(key, rd);
      if ((rd as any).updateIntervalSeconds) {
        intervals[key] = setInterval(
          () => fetchKey(key, rd),
          (rd as any).updateIntervalSeconds * 1000,
        );
      }
    }

    return () => {
      mounted = false;
      Object.values(intervals).forEach(clearInterval);
    };
  });

  // ── Context menu listener ────────────────────────────────────────────────

  const menuIdentifier = crypto.randomUUID();
  const callbackEvent = $derived(`context-menu::${self.id.replace("@", "")}`);

  $effect(() => {
    let unlistenContextMenu: (() => void) | undefined;

    Widget.self.webview
      .listen(callbackEvent, ({ payload }) => {
        const { key } = payload as any;
        if (key === "remove") {
          toolbarActions.removeItem(self.id);
        }
      })
      .then((fn) => {
        unlistenContextMenu = fn;
      });

    return () => {
      unlistenContextMenu?.();
    };
  });

  // ── Scope computation ────────────────────────────────────────────────────

  const _scopeResult = $derived(buildItemScope(self.scopes, self.id, $t));

  const fetching = $derived(_scopeResult.fetching);

  const scope = $derived.by(() => ({
    ..._scopeResult.data,
    ...fetchedData,
    trigger: (widgetId: WidgetId) => triggerWidget(widgetId, self.id),
  }));

  // ── Sandboxed code evaluation ────────────────────────────────────────────

  const content = $derived(evalSandboxedCode(self.template, scope));
  const tooltip = $derived(self.tooltip ? evalSandboxedCode(self.tooltip, scope) : null);
  const badge = $derived(self.badge ? evalSandboxedCode(self.badge, scope) : null);

  // ── Event handlers ───────────────────────────────────────────────────────

  const alignY = $derived(
    settingsState.position === FancyToolbarSide.Bottom ? Alignment.End : Alignment.Start,
  );

  const tooltipText = $derived(tooltip ? stringFromEvaluated(tooltip) : undefined);

  function handleClick() {
    if (self.onClick) {
      EvaluateAction(self.onClick, scope);
    }
  }

  function handleContextMenu(e: MouseEvent) {
    e.stopPropagation();
    const menu: ContextMenu = {
      identifier: menuIdentifier,
      items: [
        {
          type: "Item",
          key: "remove",
          label: $t("context_menu.remove"),
          icon: "CgExtensionRemove",
          callbackEvent,
        },
      ],
    };
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...menu, alignX: Alignment.Center, alignY: alignY },
      forwardTo: null,
    });
  }

  function handleWheel(e: WheelEvent) {
    const handler = e.deltaY < 0 ? self.onWheelUp : self.onWheelDown;
    if (handler) {
      EvaluateAction(handler, scope);
    }
  }

  const itemStyle = $derived(
    styleToString({
      ...self.style,
      opacity: sortable?.isDragging ? 0.3 : 1,
    }),
  );
</script>

{#if !fetching && content}
  {#if self.id.startsWith("hardcoded-separator")}
    <div {@attach sortable?.attach ?? noopAttach} class="ft-bar-separator"></div>
  {:else}
    <div
      id={self.id}
      {@attach sortable?.attach ?? noopAttach}
      role="button"
      tabindex="0"
      data-dragging={sortable?.isDragging}
      data-tooltip={tooltipText}
      data-tooltip-align-x="Center"
      data-tooltip-align-y={alignY}
      style={itemStyle}
      class="ft-bar-item"
      class:ft-bar-item-clickable={!!self.onClick}
      onclick={handleClick}
      onwheel={self.onWheelUp || self.onWheelDown ? handleWheel : undefined}
      oncontextmenu={handleContextMenu}
      onkeypress={() => {}}
    >
      <div class="ft-bar-item-content">
        <EvaluatedComponents {content} />
        {#if badge}
          <div class="ft-bar-item-badge">
            <EvaluatedComponents content={badge} />
          </div>
        {/if}
      </div>
    </div>
  {/if}
{/if}

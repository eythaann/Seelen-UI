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
  import { evalActionSanboxed, triggerWidget } from "../actionEvaluator.ts";
  import {
    compileSandboxed,
    evalComponentSandboxed,
    stringFromEvaluated,
  } from "../evaluatedComponents.ts";
  import { toolbarActions } from "../state/items.svelte.ts";
  import { settingsState } from "../state/settings.svelte.ts";
  import { styleToString } from "../utils.ts";
  import EvaluatedComponents from "../EvaluatedComponents.svelte";
  import Sandbox from "@nyariv/sandboxjs";
  import { createRemoteDataResolver } from "../remoteData.svelte.ts";
  import { resolveScopes } from "libs/ui/svelte/utils/scopes.svelte.ts";

  interface Props {
    module: ToolbarItem;
    sortable?: ReturnType<typeof createSortable> | null;
  }

  let { module: self, sortable = null }: Props = $props();

  const noopAttach = () => {};

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

  let userSourceName = $derived.by(() => {
    const allByWidget = settingsState.allByWidget;
    const userMenuConfig = allByWidget["@seelen/user-menu" as WidgetId];
    return userMenuConfig?.displayNameSource as string;
  });

  let fetchedData = createRemoteDataResolver(() => self.remoteData ?? {});

  const _scopeResult = $derived(resolveScopes(self.scopes, { userSourceName }));
  const fetching = $derived(_scopeResult.fetching);
  const scope = $derived.by(() => ({
    ..._scopeResult.data,
    ...fetchedData,
    t: (...args: [string, Record<string, string>]) => $t(...args),
  }));

  // ── Sandboxed code evaluation ────────────────────────────────────────────

  const sandbox = new Sandbox();

  const contentExec = $derived(compileSandboxed(sandbox, self.template));
  const tooltipExec = $derived(compileSandboxed(sandbox, self.tooltip));
  const badgeExec = $derived(compileSandboxed(sandbox, self.badge));

  const onClickExec = $derived(compileSandboxed(sandbox, self.onClick));
  const onWheelUpExec = $derived(compileSandboxed(sandbox, self.onWheelUp));
  const onWheelDownExec = $derived(compileSandboxed(sandbox, self.onWheelDown));

  const content = $derived(evalComponentSandboxed(self.template, contentExec, scope));
  const tooltip = $derived(
    self.tooltip ? evalComponentSandboxed(self.tooltip, tooltipExec, scope) : null,
  );
  const badge = $derived(self.badge ? evalComponentSandboxed(self.badge, badgeExec, scope) : null);

  // ── Others derives ───────────────────────────────────────────────────────

  const alignY = $derived(
    settingsState.position === FancyToolbarSide.Bottom ? Alignment.End : Alignment.Start,
  );

  const tooltipText = $derived(tooltip ? stringFromEvaluated(tooltip) : undefined);

  const itemStyle = $derived(
    styleToString({
      ...self.style,
      opacity: sortable?.isDragging ? 0.3 : 1,
    }),
  );

  // ── Event handlers ───────────────────────────────────────────────────────

  function handleClick() {
    evalActionSanboxed(onClickExec, {
      ...scope,
      trigger: (widgetId: WidgetId) => triggerWidget(widgetId, self.id),
    });
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
    evalActionSanboxed(e.deltaY < 0 ? onWheelUpExec : onWheelDownExec, {
      ...scope,
      trigger: (widgetId: WidgetId) => triggerWidget(widgetId, self.id),
    });
  }
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

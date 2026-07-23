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
  import { evalComponentSandboxed, stringFromEvaluated } from "../evaluatedComponents.ts";
  import { toolbarActions } from "../state/items.svelte.ts";
  import { settingsState } from "../state/settings.svelte.ts";
  import { styleToString } from "../utils.ts";
  import EvaluatedComponents from "../EvaluatedComponents.svelte";
  import { createRemoteDataResolver } from "../remoteData.svelte.ts";
  import { resolveScopes } from "libs/ui/svelte/utils/scopes.svelte.ts";
  import {
    compileSandboxed,
    createCanvasSandbox,
    evalSanboxed,
    getSystemTokens,
    getThemeTokens,
  } from "libs/ui/svelte/utils/sandbox.ts";
  import { prefersDarkColorScheme } from "libs/ui/svelte/runes/DarkMode.svelte.ts";

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
    position: settingsState.position,
    t: (...args: [string, Record<string, string>]) => $t(...args),
  }));

  // ── Sandboxed code evaluation ────────────────────────────────────────────

  const sandbox = createCanvasSandbox();
  let canvas = $state<HTMLCanvasElement | null>(null);

  const contentExec = $derived(compileSandboxed(sandbox, self.template));
  const renderExec = $derived(compileSandboxed(sandbox, self.render));
  const tooltipExec = $derived(compileSandboxed(sandbox, self.tooltip));
  const badgeExec = $derived(compileSandboxed(sandbox, self.badge));

  const onClickExec = $derived(compileSandboxed(sandbox, self.onClick));
  const onWheelUpExec = $derived(compileSandboxed(sandbox, self.onWheelUp));
  const onWheelDownExec = $derived(compileSandboxed(sandbox, self.onWheelDown));

  const content = $derived(self.render ? null : evalComponentSandboxed(contentExec, scope));
  const tooltip = $derived(self.tooltip ? evalComponentSandboxed(tooltipExec, scope) : null);
  const badge = $derived(self.badge ? evalComponentSandboxed(badgeExec, scope) : null);

  const canvasWidth = $derived(
    self.canvasSize ? `${self.canvasSize}px` : "var(--config-item-size)",
  );

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

  $effect(() => {
    if (!self.render || !renderExec || !canvas) return;

    canvas.width = canvas.clientWidth * window.devicePixelRatio;
    canvas.height = canvas.clientHeight * window.devicePixelRatio;

    const computed = getComputedStyle(canvas);
    evalSanboxed(renderExec, {
      ...scope,
      isDarkMode: prefersDarkColorScheme.value,
      systemTokens: getSystemTokens(computed),
      themeTokens: getThemeTokens(computed),
      canvas: {
        getContext: (contextId: string) => canvas!.getContext(contextId),
        width: canvas.width,
        height: canvas.height,
      },
    });
  });
</script>

{#if !fetching && (content || self.render)}
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
        {#if self.render}
          <canvas bind:this={canvas} class="ft-bar-item-canvas" style:width={canvasWidth}></canvas>
        {:else}
          <EvaluatedComponents {content} />
        {/if}

        {#if badge}
          <div class="ft-bar-item-badge">
            <EvaluatedComponents content={badge} />
          </div>
        {/if}
      </div>
    </div>
  {/if}
{/if}

<style>
  .ft-bar-item-canvas {
    display: block;
    height: var(--config-item-size);
  }
</style>

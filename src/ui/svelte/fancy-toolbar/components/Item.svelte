<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { Alignment, FancyToolbarSide, type ToolbarItem, type WidgetId } from "@seelen-ui/lib/types";
  import type { createSortable } from "@dnd-kit/svelte/sortable";
  import { t } from "../i18n/index.ts";
  import { evalActionSanboxed } from "./Evaluated/actionEvaluator.ts";
  import { getMenuForItem } from "../itemMenu.ts";
  import { settingsState } from "../state/settings.svelte.ts";
  import { styleToString } from "../utils.ts";
  import { createRemoteDataResolver } from "../remoteData.svelte.ts";
  import { resolveScopes } from "libs/ui/svelte/utils/scopes.svelte.ts";
  import {
    compileSandboxed,
    evalSanboxed,
    evalToStr,
    getSystemTokens,
    getThemeTokens,
  } from "libs/ui/svelte/utils/sandbox.ts";
  import { prefersDarkColorScheme } from "libs/ui/svelte/runes/DarkMode.svelte.ts";
  import { CssHandled } from "libs/ui/svelte/utils/animations.ts";
  import { evalComponentSandboxed } from "./Evaluated/definitions.ts";
  import UnknownEvaluatedComponent from "./Evaluated/Unknown.svelte";

  interface Props {
    module: ToolbarItem;
    sortable?: ReturnType<typeof createSortable> | null;
    pluginId?: string;
    placement: string;
  }

  let { module: self, sortable = null, pluginId, placement }: Props = $props();

  const noopAttach = () => {};

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
    self: { placement },
    position: settingsState.position, // @deprecated remove after v3
    toolbar: { position: settingsState.position },
    t: (...args: [string, Record<string, string>]) => $t(...args),
  }));

  // ── Sandboxed code evaluation ────────────────────────────────────────────

  let canvas = $state<HTMLCanvasElement | null>(null);

  const contentExec = $derived(compileSandboxed(self.template));
  const renderExec = $derived(compileSandboxed(self.render));
  const tooltipExec = $derived(compileSandboxed(self.tooltip));
  const badgeExec = $derived(compileSandboxed(self.badge));

  const onClickExec = $derived(compileSandboxed(self.onClick));
  const onWheelUpExec = $derived(compileSandboxed(self.onWheelUp));
  const onWheelDownExec = $derived(compileSandboxed(self.onWheelDown));

  const content = $derived(self.render ? null : evalComponentSandboxed(contentExec, scope));
  const tooltip = $derived(evalToStr(evalComponentSandboxed(tooltipExec, scope)));
  const badge = $derived(evalToStr(evalComponentSandboxed(badgeExec, scope)));

  const canvasWidth = $derived(
    self.canvasSize ? `${self.canvasSize}px` : "var(--config-item-size)",
  );

  // ── Others derives ───────────────────────────────────────────────────────

  const itemStyle = $derived(
    styleToString({
      ...self.style,
      ...(sortable?.isDragging ? { opacity: 0.2 } : {}),
    }),
  );

  // ── Event handlers ───────────────────────────────────────────────────────

  function handleClick() {
    evalActionSanboxed(self.id, onClickExec, scope);
  }

  function handleContextMenu(e: MouseEvent) {
    e.stopPropagation();
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...getMenuForItem($t, self.id), alignX: Alignment.Center, alignY: settingsState.popupAlignY },
      forwardTo: null,
    });
  }

  function handleWheel(e: WheelEvent) {
    evalActionSanboxed(self.id, e.deltaY < 0 ? onWheelUpExec : onWheelDownExec, scope);
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
      data-plugin-id={pluginId}
      data-dragging={sortable?.isDragging}
      data-tooltip={tooltip}
      data-tooltip-align-x="Center"
      data-tooltip-align-y={settingsState.popupAlignY}
      data-tooltip-origin-y={settingsState.tooltipY}
      style={itemStyle}
      class="ft-bar-item"
      class:ft-bar-item-clickable={!!self.onClick}
      onclick={handleClick}
      onwheel={self.onWheelUp || self.onWheelDown ? handleWheel : undefined}
      oncontextmenu={handleContextMenu}
      onkeypress={() => {}}
      transition:CssHandled|global={{
        enabled() {
          return !!sortable && !sortable.isDragging;
        },
      }}
    >
      <div class="ft-bar-item-content">
        {#if self.render}
          <canvas bind:this={canvas} class="ft-bar-item-canvas" style:width={canvasWidth}></canvas>
        {:else}
          <UnknownEvaluatedComponent parentId={self.id} {content} />
        {/if}
      </div>

      {#if badge}
        <div class="ft-bar-item-badge" transition:CssHandled>{badge}</div>
      {/if}
    </div>
  {/if}
{/if}

<style>
  .ft-bar-item-canvas {
    display: block;
    height: var(--config-item-size);
  }
</style>

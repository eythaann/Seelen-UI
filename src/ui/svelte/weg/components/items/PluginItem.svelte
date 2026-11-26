<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { CanvasSize, type WegPluginItem as WegPluginPayload, type WidgetId } from "@seelen-ui/lib/types";
  import type { PluginWegItem } from "../../types.ts";
  import { t } from "../../i18n/index.ts";
  import { getEditCustomIconEntry, getEmptyTrashBinEntry, getMenuForItem } from "../../generalMenu.ts";
  import { evalActionSanboxed, triggerWidget } from "../../pluginEval.svelte.ts";
  import { resolveScopes } from "libs/ui/svelte/utils/scopes.svelte.ts";
  import { prefersDarkColorScheme } from "libs/ui/svelte/runes/DarkMode.svelte.ts";
  import { SpecificIcon } from "libs/ui/svelte/components/Icon/index.ts";

  import { settingsState } from "../../state/settings.svelte.ts";
  import { compileSandboxed, drawPluginCanvas, evalSanboxed, evalToStr } from "libs/ui/svelte/utils/sandbox.ts";

  interface Props {
    item: PluginWegItem;
    payload: WegPluginPayload;
  }

  let { item, payload }: Props = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let canvasWidth = $state(0);
  let canvasHeight = $state(0);

  let userSourceName = $derived.by(() => {
    const allByWidget = settingsState.allByWidget;
    const userMenuConfig = allByWidget["@seelen/user-menu" as WidgetId];
    return userMenuConfig?.displayNameSource as string;
  });

  const scopeResult = $derived(resolveScopes(payload.scopes, { userSourceName }));
  const scope = $derived({
    ...scopeResult.data,
    position: settingsState.position, // @deprecated remove after v3
    dock: { position: settingsState.position },
    t: (...args: [string, Record<string, string>]) => $t(...args),
  });

  const renderExec = $derived(compileSandboxed(payload.render));
  const tooltipExec = $derived(compileSandboxed(payload.tooltip));
  const badgeExec = $derived(compileSandboxed(payload.badge));
  const onClickExec = $derived(compileSandboxed(payload.onClick));

  const tooltipText = $derived(evalToStr(evalSanboxed(tooltipExec, scope)));
  const badgeText = $derived(evalToStr(evalSanboxed(badgeExec, scope)));

  const customIconKey = $derived(
    payload.noCanvas ? evalToStr(evalSanboxed(renderExec, scope)) : null,
  );

  const hasTrashBinScope = $derived(payload.scopes.some((s) => s.toLowerCase() === "trashbin"));

  function handleClick() {
    evalActionSanboxed(onClickExec, {
      trigger: (widgetId: WidgetId) => triggerWidget(widgetId, item.id),
    });
  }

  function handleContextMenu(e: MouseEvent) {
    e.stopPropagation();
    const alignX = settingsState.popupAlignX;
    const alignY = settingsState.popupAlignY;

    const menu = getMenuForItem($t, item);

    if (customIconKey) {
      menu.items.unshift(getEditCustomIconEntry($t, customIconKey), { type: "Separator" });
    }

    if (hasTrashBinScope) {
      menu.items.unshift(getEmptyTrashBinEntry($t), { type: "Separator" });
    }

    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...menu, alignX, alignY },
      forwardTo: null,
    });
  }

  $effect(() => {
    const target = canvas;
    const executor = renderExec;
    if (payload.noCanvas || !target || !executor) return;

    // Theme settings can change the tokens even when the scope data is unchanged.
    void settingsState.all;
    const width = canvasWidth;
    const height = canvasHeight;
    const renderScope = scope;
    const isDarkMode = prefersDarkColorScheme.value;
    const frame = requestAnimationFrame(() =>
      drawPluginCanvas(target, width, height, executor, renderScope, isDarkMode)
    );
    return () => cancelAnimationFrame(frame);
  });
</script>

{#if !scopeResult.fetching}
  <div class="weg-item-overlay">
    <div
      id={item.id}
      role="button"
      tabindex="0"
      data-plugin-id={item.plugin}
      class="weg-item"
      class:weg-item-medium={payload.canvasSize === CanvasSize.Medium}
      class:weg-item-large={payload.canvasSize === CanvasSize.Large}
      data-tooltip={tooltipText}
      data-tooltip-origin-y={settingsState.tooltipOrigin.y}
      data-tooltip-origin-x={settingsState.tooltipOrigin.x}
      data-tooltip-align-x={settingsState.popupAlignX}
      data-tooltip-align-y={settingsState.popupAlignY}
      onclick={handleClick}
      oncontextmenu={handleContextMenu}
      onkeypress={() => {}}
    >
      {#if payload.noCanvas}
        <SpecificIcon class="weg-item-icon" name={customIconKey || ""} />
      {:else}
        <canvas
          bind:this={canvas}
          bind:clientWidth={canvasWidth}
          bind:clientHeight={canvasHeight}
          class="weg-item-canvas"
        ></canvas>
      {/if}
    </div>

    {#if badgeText}
      <div class="weg-item-badge">{badgeText}</div>
    {/if}
  </div>
{/if}

<style>
  .weg-item-canvas {
    width: 100%;
    height: 100%;
  }
</style>

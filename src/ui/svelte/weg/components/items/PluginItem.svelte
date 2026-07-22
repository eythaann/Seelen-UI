<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import {
    type WegPluginItem as WegPluginPayload,
    type WidgetId,
    CanvasSize,
  } from "@seelen-ui/lib/types";
  import type { PluginWegItem } from "../../types.ts";
  import { t } from "../../i18n/index.ts";
  import {
    getEditCustomIconEntry,
    getEmptyTrashBinEntry,
    getMenuForItem,
  } from "../../generalMenu.ts";
  import {
    compileSandboxed,
    createPluginSandbox,
    evalActionSanboxed,
    evalSanboxed,
    stringFromEvaluated,
    triggerWidget,
  } from "../../pluginEval.svelte.ts";
  import { resolveScopes } from "libs/ui/svelte/utils/scopes.svelte.ts";
  import { prefersDarkColorScheme } from "libs/ui/svelte/runes/DarkMode.svelte.ts";
  import { SpecificIcon } from "libs/ui/svelte/components/Icon/index.ts";

  import { settingsState } from "../../state/settings.svelte.ts";

  interface Props {
    item: PluginWegItem;
    payload: WegPluginPayload;
  }

  let { item, payload }: Props = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);

  let userSourceName = $derived.by(() => {
    const allByWidget = settingsState.allByWidget;
    const userMenuConfig = allByWidget["@seelen/user-menu" as WidgetId];
    return userMenuConfig?.displayNameSource as string;
  });

  const scopeResult = $derived(resolveScopes(payload.scopes, { userSourceName }));
  const scope = $derived({
    ...scopeResult.data,
    position: settingsState.position,
    t: (...args: [string, Record<string, string>]) => $t(...args),
  });

  const sandbox = createPluginSandbox();

  const renderExec = $derived(compileSandboxed(sandbox, payload.render));
  const tooltipExec = $derived(compileSandboxed(sandbox, payload.tooltip));
  const badgeExec = $derived(compileSandboxed(sandbox, payload.badge));
  const onClickExec = $derived(compileSandboxed(sandbox, payload.onClick));

  const tooltipText = $derived(
    payload.tooltip ? stringFromEvaluated(evalSanboxed(tooltipExec, scope)) : null,
  );

  const badgeText = $derived(
    payload.badge ? stringFromEvaluated(evalSanboxed(badgeExec, scope)) : null,
  );

  const customIconKey = $derived(
    payload.noCanvas ? stringFromEvaluated(evalSanboxed(renderExec, scope)) : null,
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
    if (payload.noCanvas || !renderExec || !canvas) return;

    canvas.width = canvas.clientWidth * window.devicePixelRatio;
    canvas.height = canvas.clientHeight * window.devicePixelRatio;

    const computed = getComputedStyle(canvas);
    evalSanboxed(renderExec, {
      ...scope,
      isDarkMode: prefersDarkColorScheme.value,
      systemTokens: {
        accentLightestColor: computed.getPropertyValue("--system-accent-lightest-color"),
        accentLighterColor: computed.getPropertyValue("--system-accent-lighter-color"),
        accentLightColor: computed.getPropertyValue("--system-accent-light-color"),
        accentColor: computed.getPropertyValue("--system-accent-color"),
        accentDarkColor: computed.getPropertyValue("--system-accent-dark-color"),
        accentDarkerColor: computed.getPropertyValue("--system-accent-darker-color"),
        accentDarkestColor: computed.getPropertyValue("--system-accent-darkest-color"),
      },
      themeTokens: {
        foregroundColor: computed.getPropertyValue("--slu-std-fg-color"),
        foregroundSecondaryColor: computed.getPropertyValue("--slu-std-fg-secondary-color"),
        foregroundMutedColor: computed.getPropertyValue("--slu-std-fg-muted-color"),
        foregroundDisabledColor: computed.getPropertyValue("--slu-std-fg-disabled-color"),
        backgroundColor: computed.getPropertyValue("--slu-std-bg-color"),
      },
      canvas: {
        getContext: (contextId: string) => canvas!.getContext(contextId),
        width: canvas.width,
        height: canvas.height,
      },
    });

    // note: <img> downscaling uses the browser's high-quality resampler; a canvas
    // rendered at CANVAS_SIZE (256) and shrunk via CSS looks noticeably pixelated.
    // this was before but now we use canvas directly because performance on fast changes.
  });
</script>

{#if !scopeResult.fetching}
  <div class="weg-item-overlay">
    <div
      id={item.id}
      role="button"
      tabindex="0"
      class="weg-item"
      class:weg-item-medium={payload.canvasSize === CanvasSize.Medium}
      class:weg-item-large={payload.canvasSize === CanvasSize.Large}
      data-tooltip={tooltipText}
      data-tooltip-align-x={settingsState.popupAlignX}
      data-tooltip-align-y={settingsState.popupAlignY}
      onclick={handleClick}
      oncontextmenu={handleContextMenu}
      onkeypress={() => {}}
    >
      {#if payload.noCanvas}
        <SpecificIcon class="weg-item-icon" name={customIconKey || ""} />
      {:else}
        <canvas bind:this={canvas} class="weg-item-canvas"></canvas>
      {/if}
    </div>

    {#if badgeText}
      <div class="weg-item-custom-badge">{badgeText}</div>
    {/if}
  </div>
{/if}

<style>
  .weg-item-canvas {
    width: 100%;
    height: 100%;
  }
</style>

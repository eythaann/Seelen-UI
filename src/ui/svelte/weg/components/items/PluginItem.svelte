<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import type { WegPluginItem as WegPluginPayload, WidgetId } from "@seelen-ui/lib/types";
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

  const CANVAS_SIZE = 256;

  interface Props {
    item: PluginWegItem;
    payload: WegPluginPayload;
  }

  let { item, payload }: Props = $props();

  let img = $state<HTMLImageElement | null>(null);
  const canvas = document.createElement("canvas");
  canvas.width = CANVAS_SIZE;
  canvas.height = CANVAS_SIZE;
  let lastObjectUrl: string | null = null;

  let userSourceName = $derived.by(() => {
    const allByWidget = settingsState.allByWidget;
    const userMenuConfig = allByWidget["@seelen/user-menu" as WidgetId];
    return userMenuConfig?.displayNameSource as string;
  });

  const scopeResult = $derived(resolveScopes(payload.scopes, { userSourceName }));
  const scope = $derived({
    ...scopeResult.data,
    position: settingsState.position,
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
    if (payload.noCanvas || !renderExec || !img) return;

    const computed = getComputedStyle(img);
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
        getContext: (contextId: string) => canvas.getContext(contextId),
        width: CANVAS_SIZE,
        height: CANVAS_SIZE,
      },
    });

    // <img> downscaling uses the browser's high-quality resampler; a canvas
    // rendered at CANVAS_SIZE and shrunk via CSS looks noticeably pixelated.
    canvas.toBlob((blob) => {
      if (!blob || !img) return;
      const url = URL.createObjectURL(blob);
      img.src = url;
      if (lastObjectUrl) URL.revokeObjectURL(lastObjectUrl);
      lastObjectUrl = url;
    });
  });
</script>

{#if !scopeResult.fetching}
  <div class="weg-item-overlay">
    <div
      id={item.id}
      role="button"
      tabindex="0"
      class="weg-item"
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
        <img bind:this={img} class="weg-item-icon" alt="" />
      {/if}
    </div>

    {#if badgeText}
      <div class="weg-item-custom-badge">{badgeText}</div>
    {/if}
  </div>
{/if}

<style>
  .weg-item-icon {
    width: 100%;
    height: 100%;
  }
</style>

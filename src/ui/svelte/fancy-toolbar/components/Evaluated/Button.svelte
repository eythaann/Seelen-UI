<script lang="ts">
  import { styleToString } from "../../utils";
  import type { EvaluatedButtonProps } from "./definitions";
  import { compileSandboxed } from "libs/ui/svelte/utils/sandbox";
  import { evalActionSanboxed } from "./actionEvaluator";
  import Unknown from "./Unknown.svelte";
  import { settingsState } from "../../state/settings.svelte";

  let {
    parentId,
    style,
    content,
    tooltip,
    onClick,
    onDoubleClick,
    onAuxClick,
    onContextMenu,
  }: EvaluatedButtonProps & { parentId: string } = $props();

  let styleString = $derived(styleToString(style));

  const onClickExec = $derived(compileSandboxed(onClick));
  const onDoubleClickExec = $derived(compileSandboxed(onDoubleClick));
  const onAuxClickExec = $derived(compileSandboxed(onAuxClick));
  const onContextMenuExec = $derived(compileSandboxed(onContextMenu));
</script>

<button
  data-skin="transparent"
  data-tooltip={tooltip}
  data-tooltip-align-x="Center"
  data-tooltip-align-y={settingsState.popupAlignY}
  data-tooltip-origin-y={settingsState.tooltipY}
  style={styleString}
  onclick={(e) => {
    // the second click of a double click is handled by `onDoubleClick`
    if (onDoubleClickExec && e.detail === 2) {
      return;
    }
    if (onClickExec) {
      evalActionSanboxed(parentId, onClickExec, {});
    }
  }}
  ondblclick={() => {
    if (onDoubleClickExec) {
      evalActionSanboxed(parentId, onDoubleClickExec, {});
    }
  }}
  onauxclick={(e) => {
    if (e.button !== 1) {
      return;
    }
    if (onAuxClickExec) {
      evalActionSanboxed(parentId, onAuxClickExec, {});
    }
  }}
  oncontextmenu={(e) => {
    if (onContextMenuExec) {
      e.stopPropagation(); // avoid opening the context menu of the module/item
      evalActionSanboxed(parentId, onContextMenuExec, {});
    }
  }}
>
  <Unknown {parentId} {content} />
</button>

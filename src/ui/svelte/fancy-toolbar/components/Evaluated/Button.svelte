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
    onAuxClick,
    onContextMenu,
  }: EvaluatedButtonProps & { parentId: string } = $props();

  let styleString = $derived(styleToString(style));

  const onClickExec = $derived(compileSandboxed(onClick));
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
    if (onClickExec) {
      evalActionSanboxed(parentId, onClickExec, {});
    }
  }}
  onauxclick={(e) => {
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

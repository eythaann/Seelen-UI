<script lang="ts">
  import Sandbox from "@nyariv/sandboxjs";
  import { styleToString } from "../../utils";
  import type { EvaluatedButtonProps } from "./definitions";
  import { compileSandboxed } from "libs/ui/svelte/utils/sandbox";
  import { evalActionSanboxed } from "./actionEvaluator";
  import Unknown from "./Unknown.svelte";

  let {
    parentId,
    style,
    content,
    onClick,
    onAuxClick,
    onContextMenu,
  }: EvaluatedButtonProps & { parentId: string } = $props();

  let styleString = $derived(styleToString(style));

  const sandbox = new Sandbox();
  const onClickExec = $derived(compileSandboxed(sandbox, onClick));
  const onAuxClickExec = $derived(compileSandboxed(sandbox, onAuxClick));
  const onContextMenuExec = $derived(compileSandboxed(sandbox, onContextMenu));
</script>

<button
  data-skin="transparent"
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

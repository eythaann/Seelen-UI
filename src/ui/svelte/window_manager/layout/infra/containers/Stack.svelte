<script lang="ts">
  import type { WmNode } from "@seelen-ui/lib/types";
  import { state } from "../../../state.svelte.ts";
  import Leaf from "./Leaf.svelte";
  import FileIcon from "libs/ui/svelte/components/Icon/FileIcon.svelte";

  interface Props {
    node: WmNode;
    overlayVisible: boolean;
  }

  let { node, overlayVisible }: Props = $props();
</script>

<div style:flex-grow={node.growFactor} class={["wm-container", "wm-stack"]}>
  {#if node.windows.length > 1}
    <div class="wm-stack-bar" data-allow-mouse-events={overlayVisible}>
      {#each node.windows as winId (winId)}
        {@const info = state.interactables.find((app) => app.hwnd === winId)}
        <div
          class={[
            "wm-stack-bar-item",
            {
              "wm-stack-bar-item-active": winId === node.active,
            },
          ]}
          data-allow-mouse-events={overlayVisible}
        >
          <FileIcon
            path={info?.process?.path}
            umid={info?.umid}
            class="wm-stack-bar-item-icon"
            data-allow-mouse-events={overlayVisible}
          />
          <span class="wm-stack-bar-item-title" data-allow-mouse-events={overlayVisible}>
            {info?.title || `0x${winId.toString(16)}`}
          </span>
        </div>
      {/each}
    </div>
  {/if}
  {#if node.active}
    <Leaf hwnd={node.active} />
  {/if}
</div>

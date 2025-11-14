<script lang="ts">
  import type { WmNode } from "@seelen-ui/lib/types";
  import { state } from "../../../shared/state.svelte.ts";
  import Leaf from "./Leaf.svelte";
  import FileIcon from "libs/ui/svelte/components/Icon/FileIcon.svelte";

  interface Props {
    node: WmNode;
  }

  let { node }: Props = $props();
</script>

<div style:flex-grow={node.growFactor} class={["wm-container", "wm-stack"]}>
  {#if node.windows.length > 1}
    <div class="wm-stack-bar" data-allow-mouse-events={state.overlayVisible}>
      {#each node.windows as handle (handle)}
        {@const info = state.openApps.find((app) => app.windows.some((w) => w.handle === handle))}
        <div
          class={[
            "wm-stack-bar-item",
            {
              "wm-stack-bar-item-active": handle === node.active,
            },
          ]}
          data-allow-mouse-events={state.overlayVisible}
        >
          <FileIcon
            path={info?.path}
            umid={info?.umid}
            class="wm-stack-bar-item-icon"
            data-allow-mouse-events={state.overlayVisible}
          />
          <span class="wm-stack-bar-item-title" data-allow-mouse-events={state.overlayVisible}>
            {info?.windows.find((w) => w.handle === handle)?.title || `0x${handle.toString(16)}`}
          </span>
        </div>
      {/each}
    </div>
  {/if}
  {#if node.active}
    <Leaf hwnd={node.active} />
  {/if}
</div>

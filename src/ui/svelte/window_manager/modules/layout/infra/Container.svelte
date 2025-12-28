<script lang="ts">
  import { WmNodeKind } from "@seelen-ui/lib/types";
  import type { Node } from "../domain.ts";
  import { NodeUtils } from "../../shared/utils.ts";
  import Leaf from "./containers/Leaf.svelte";
  import Stack from "./containers/Stack.svelte";
  import Container from "./Container.svelte";

  interface Props {
    node: Node;
  }

  let { node }: Props = $props();
</script>

{#if !NodeUtils.isEmpty(node)}
  {#if node.type === WmNodeKind.Stack}
    <Stack {node} />
  {:else if node.type === WmNodeKind.Leaf && node.active}
    <Leaf hwnd={node.active} growFactor={node.growFactor} />
  {:else if node.type === WmNodeKind.Horizontal || node.type === WmNodeKind.Vertical}
    <div
      style:flex-grow={node.growFactor}
      class={["wm-container", `wm-${node.type.toLowerCase()}`]}
    >
      {#each node.children as child, idx (idx)}
        <Container node={child} />
      {/each}
    </div>
  {/if}
{/if}

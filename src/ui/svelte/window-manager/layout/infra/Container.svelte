<script lang="ts">
  import { getContext } from "svelte";
  import type { TwmRuntimeTree } from "@seelen-ui/lib/types";
  import { TwmNodeKind } from "@seelen-ui/lib/types";
  import { NodeUtils } from "../../utils.ts";
  import { TREE_CONTEXT_KEY } from "../domain.ts";
  import Leaf from "./containers/Leaf.svelte";
  import Stack from "./containers/Stack.svelte";
  import Container from "./Container.svelte";

  interface Props {
    nodeId: number;
    stackBarInteractive: boolean;
  }

  let { nodeId, stackBarInteractive }: Props = $props();

  const ctx = getContext<{ tree: TwmRuntimeTree | null }>(TREE_CONTEXT_KEY);
  let tree = $derived(ctx.tree);
  let node = $derived(tree?.nodes[nodeId]);
</script>

{#if node && tree && !NodeUtils.isEmpty(tree, nodeId)}
  {#if node.kind === TwmNodeKind.Stack}
    <Stack {node} {stackBarInteractive} />
  {:else if node.kind === TwmNodeKind.Leaf && node.activeWindow !== null}
    <Leaf hwnd={node.activeWindow} growFactor={node.growFactor} />
  {:else if node.kind === TwmNodeKind.Horizontal || node.kind === TwmNodeKind.Vertical}
    <div
      style:flex-grow={node.growFactor}
      class={["wm-container", `wm-${node.kind.toLowerCase()}`]}
    >
      {#each node.children as childId (childId)}
        <Container nodeId={childId} {stackBarInteractive} />
      {/each}
    </div>
  {/if}
{/if}

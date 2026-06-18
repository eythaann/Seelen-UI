<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { IconName } from "libs/ui/icons";
  import { Icon, FileIcon } from "libs/ui/svelte/components/Icon";
  import { EvaluateAction } from "./actionEvaluator.ts";
  import { ObjectComponentKind, parseComponentProps } from "./evaluatedComponents.ts";
  import { styleToString } from "./utils.ts";
  import EvaluatedComponents from "./EvaluatedComponents.svelte";

  interface Props {
    content: unknown;
  }

  let { content }: Props = $props();
</script>

{#if typeof content === "string"}
  <span>{content}</span>
{:else if typeof content === "number" || typeof content === "boolean" || typeof content === "bigint"}
  <span>{String(content)}</span>
{:else if Array.isArray(content)}
  {#each content as item, i (i)}
    <EvaluatedComponents content={item} />
  {/each}
{:else if content !== null && typeof content === "object"}
  {@const parsed = parseComponentProps(content as object)}
  {#if parsed?.kind === ObjectComponentKind.Icon}
    <Icon iconName={parsed.props.name as IconName} />
  {:else if parsed?.kind === ObjectComponentKind.AppIcon}
    <FileIcon path={parsed.props.path} umid={parsed.props.umid} />
  {:else if parsed?.kind === ObjectComponentKind.Image}
    <img
      src={parsed.props.path ? convertFileSrc(parsed.props.path) : parsed.props.url || ""}
      alt=""
    />
  {:else if parsed?.kind === ObjectComponentKind.Button}
    <button
      data-skin="transparent"
      style={styleToString(parsed.props.style)}
      onclick={() => parsed.props.onClick && EvaluateAction(parsed.props.onClick, {})}
    >
      <EvaluatedComponents content={parsed.props.content} />
    </button>
  {:else if parsed?.kind === ObjectComponentKind.Group}
    <div style={styleToString(parsed.props.style)}>
      <EvaluatedComponents content={parsed.props.content} />
    </div>
  {/if}
{/if}

<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { IconName } from "libs/ui/icons";
  import { Icon, FileIcon } from "libs/ui/svelte/components/Icon";
  import { evalActionSanboxed } from "./actionEvaluator.ts";
  import { ObjectComponentKind, parseComponentProps } from "./evaluatedComponents.ts";
  import { styleToString } from "./utils.ts";
  import EvaluatedComponents from "./EvaluatedComponents.svelte";
  import Sandbox from "@nyariv/sandboxjs";

  interface Props {
    content: unknown;
  }

  let { content }: Props = $props();

  let onClickSource = $derived.by(() => {
    if (typeof content === "object") {
      const parsed = parseComponentProps(content as object);
      if (parsed?.kind === ObjectComponentKind.Button) {
        return parsed.props.onClick;
      }
    }
    return undefined;
  });
</script>

{#if typeof content === "string"}
  <span>{content}</span>
{:else if typeof content === "number" || typeof content === "boolean" || typeof content === "bigint"}
  <span>{String(content)}</span>
{:else if Array.isArray(content)}
  {#each content as item, idx (idx)}
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
      onclick={() => {
        if (!onClickSource) return;
        const sandbox = new Sandbox();
        const executor = sandbox.compileAsync(onClickSource);
        evalActionSanboxed(executor, {});
      }}
    >
      <EvaluatedComponents content={parsed.props.content} />
    </button>
  {:else if parsed?.kind === ObjectComponentKind.Group}
    <div style={styleToString(parsed.props.style)}>
      <EvaluatedComponents content={parsed.props.content} />
    </div>
  {/if}
{/if}

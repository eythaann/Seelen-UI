<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { IconName } from "libs/ui/icons";
  import { Icon, FileIcon } from "libs/ui/svelte/components/Icon";
  import { styleToString } from "../../utils.ts";
  import Unknown from "./Unknown.svelte";
  import { ObjectComponentKind, parseComponent } from "./definitions.ts";
  import Button from "./Button.svelte";

  interface Props {
    parentId: string;
    content: unknown;
  }

  let { content, parentId }: Props = $props();

  const parsed = $derived(parseComponent(content));

  function imageSrc(props: { path?: string | null; url?: string | null; version?: string | number | null }): string {
    const src = props.path ? convertFileSrc(props.path) : props.url || "";
    if (!src || props.version == null || props.version === "") {
      return src;
    }
    return `${src}${src.includes("?") ? "&" : "?"}v=${encodeURIComponent(props.version)}`;
  }
</script>

{#if parsed}
  {#if parsed.kind === ObjectComponentKind.Plain}
    <span>{parsed.value}</span>
  {:else if parsed.kind === ObjectComponentKind.PlainList}
    {#each parsed.value as entry, idx (idx)}
      <Unknown {parentId} content={entry} />
    {/each}
  {:else if parsed.kind === ObjectComponentKind.Group}
    <div style={styleToString(parsed.props.style)}>
      <Unknown {parentId} content={parsed.props.content} />
    </div>
  {:else if parsed.kind === ObjectComponentKind.Icon}
    <Icon iconName={parsed.props.name as IconName} />
  {:else if parsed.kind === ObjectComponentKind.AppIcon}
    <FileIcon path={parsed.props.path} umid={parsed.props.umid} />
  {:else if parsed.kind === ObjectComponentKind.Image}
    <img src={imageSrc(parsed.props)} alt="" />
  {:else if parsed.kind === ObjectComponentKind.Button}
    <Button {parentId} {...parsed.props} />
  {/if}
{/if}

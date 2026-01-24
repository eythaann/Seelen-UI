<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import FileIcon from "libs/ui/svelte/components/Icon/FileIcon.svelte";

  interface Props {
    path: string;
    displayName: string;
  }

  let { path, displayName }: Props = $props();

  function selectFileOnExplorer() {
    invoke(SeelenCommand.SelectFileOnExplorer, { path });
  }
</script>

<button class="user-file" onclick={selectFileOnExplorer} title={path}>
  <FileIcon class="user-file-icon" {path} lazy />
  <div class="user-file-label">{displayName}</div>
  <!-- <div class="user-file-date">{lastAccessTime}</div> -->
</button>

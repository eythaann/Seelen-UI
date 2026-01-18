<script lang="ts">
  import type { File } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import FileIcon from "libs/ui/svelte/components/Icon/FileIcon.svelte";
  import { WindowsDateFileTimeToDate, relativeTimeFromNow } from "libs/ui/svelte/utils";

  interface Props {
    file: File;
  }

  let { file }: Props = $props();

  const fileName = $derived(file.path.substring(file.path.lastIndexOf("\\") + 1));
  const lastAccessTime = $derived(relativeTimeFromNow(WindowsDateFileTimeToDate(file.lastAccessTime)));

  function selectFileOnExplorer() {
    invoke(SeelenCommand.SelectFileOnExplorer, { path: file.path });
  }
</script>

<button
  class="user-file"
  onclick={selectFileOnExplorer}
  title={file.path}
>
  <FileIcon class="user-file-icon" path={file.path} />
  <div class="user-file-label">{fileName}</div>
  <div class="user-file-date">{lastAccessTime}</div>
</button>

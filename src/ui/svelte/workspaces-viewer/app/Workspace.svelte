<script lang="ts">
  import type { DesktopWorkspace } from "@seelen-ui/lib/types";

  interface Props {
    index: number;
    workspace: DesktopWorkspace;
  }

  let { workspace, index }: Props = $props();
</script>

<div class="workspace">
  <div class="workspace-header">
    <span class="workspace-name">{workspace.name || `Desktop ${index + 1}`}</span>
    <button class="workspace-destroy">X</button>
  </div>

  <div class="workspace-preview"></div>
</div>

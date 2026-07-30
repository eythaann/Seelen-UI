<script lang="ts">
  import { Icon } from "libs/ui/svelte/components/Icon";
  import type { DesktopWorkspace } from "@seelen-ui/lib/types";

  interface Props {
    workspace: DesktopWorkspace & { rowIdx: number; colIdx: number };
  }

  let { workspace }: Props = $props();
</script>

<div class="workspace">
  <span class="workspace-name">
    {workspace.name || `Workspace ${workspace.rowIdx + 1}.${workspace.colIdx + 1}`}
  </span>
  <Icon iconName={(workspace.icon as any) || "PiMonitorBold"} />
</div>

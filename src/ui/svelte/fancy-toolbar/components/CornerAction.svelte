<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { t } from "../i18n/index.ts";
</script>

<button
  class="ft-corner-button"
  onclick={() => invoke(SeelenCommand.ShowDesktop)}
  title={$t("show_desktop")}
></button>

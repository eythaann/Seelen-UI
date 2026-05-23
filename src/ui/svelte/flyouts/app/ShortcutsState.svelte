<script lang="ts">
  import { t } from "../i18n";

  interface Props {
    paused: boolean;
  }

  let { paused }: Props = $props();
</script>

<div class="shortcuts-state">
  <span class="shortcuts-state-label">{paused ? $t("shortcuts_paused") : $t("shortcuts_resumed")}</span>
</div>

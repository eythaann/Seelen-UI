<script lang="ts">
  import { Reservation } from "../../domain.ts";
  import { state } from "../../../shared/state.svelte.ts";

  interface Props {
    reservation: Reservation;
  }

  let { reservation }: Props = $props();

  // Svelte 5 runes: $derived for computed values
  let floating = $derived(state.settings.floating);
</script>

<div
  class={["wm-container", "wm-reserved", `wm-reserved-${reservation.toLowerCase()}`]}
  style:width={reservation === Reservation.Float ? floating.width : undefined}
  style:height={reservation === Reservation.Float ? floating.height : undefined}
></div>

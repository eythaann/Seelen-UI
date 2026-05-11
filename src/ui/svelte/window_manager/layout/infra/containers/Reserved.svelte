<script lang="ts">
  import { TwmReservation } from "@seelen-ui/lib/types";
  import { state } from "../../../state.svelte.ts";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { ReservationIcon } from "../../domain.ts";

  interface Props {
    reservation: TwmReservation;
  }

  let { reservation }: Props = $props();

  let floating = $derived(state.settings.floating);
</script>

<div
  class={["wm-container", "wm-reserved", `wm-reserved-${reservation.toLowerCase()}`]}
  style:width={reservation === TwmReservation.Float ? `${floating.width}px` : undefined}
  style:height={reservation === TwmReservation.Float ? `${floating.height}px` : undefined}
>
  <Icon iconName={ReservationIcon[reservation]!} />
</div>

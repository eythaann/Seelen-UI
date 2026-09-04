<script lang="ts">
  import type { MediaPlayer } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { nanosecondsToPlayingTime } from "libs/ui/utils";
  import { throttle } from "lodash";

  interface Props {
    session: MediaPlayer;
  }

  let { session }: Props = $props();

  const NS_PER_SECOND = 1_000_000_000;
  const NS_PER_MS = 1_000_000;

  let isSeeking = $state(false);
  let seekingPositionSeconds = $state(0);

  // The backend only pushes a new `timeline.position` when it changes on the OS side
  // (which happens infrequently), so while playing we interpolate locally using the
  // elapsed wall-clock time since the last snapshot instead of freezing the display.
  let basePosition = $state(0);
  let baseTimestamp = $state(Date.now());
  let now = $state(Date.now());

  // reset state
  $effect(() => {
    basePosition = session.timeline.position;
    const timestamp = Date.now();
    baseTimestamp = timestamp;
    now = timestamp;
  });

  $effect(() => {
    if (!session.playing) return;
    const interval = setInterval(() => {
      now = Date.now();
    }, 250);
    return () => clearInterval(interval);
  });

  const livePosition = $derived(
    session.playing ? basePosition + (now - baseTimestamp) * NS_PER_MS : basePosition,
  );

  const position = $derived(
    isSeeking
      ? seekingPositionSeconds * NS_PER_SECOND
      : Math.min(livePosition, session.timeline.end),
  );

  const onSeek = throttle((positionSeconds: number) => {
    const position = Math.round(positionSeconds * NS_PER_SECOND);
    invoke(SeelenCommand.MediaSeek, { id: session.umid, position }).catch(console.error);
  }, 200);

  function onSeekInput(e: Event & { currentTarget: HTMLInputElement }) {
    isSeeking = true;
    seekingPositionSeconds = Number(e.currentTarget.value);
    onSeek(seekingPositionSeconds);
  }

  function onSeekCommit() {
    isSeeking = false;
  }
</script>

<div class="media-session-progress">
  <span class="media-session-time">{nanosecondsToPlayingTime(position)}</span>
  <input
    type="range"
    class="media-session-progress-bar"
    min={Math.floor(session.timeline.start / NS_PER_SECOND)}
    max={Math.ceil(session.timeline.end / NS_PER_SECOND)}
    step={1}
    value={Math.round(position / NS_PER_SECOND)}
    oninput={onSeekInput}
    onchange={onSeekCommit}
  />
  <span class="media-session-time">{nanosecondsToPlayingTime(session.timeline.end)}</span>
</div>

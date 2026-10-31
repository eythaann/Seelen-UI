<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { MediaTransportAction, type UserAppWindow } from "@seelen-ui/lib/types";
  import MediaTransport from "libs/ui/svelte/components/MediaTransport.svelte";
  import { t } from "libs/ui/svelte/utils/i18n.ts";
  import { previewState } from "./state.svelte.ts";
  import { browsers, executableName, isMediaShortcut, knownPlayers, matchingSessions, mediaShortcutProfile } from "./media.ts";

  let { window }: { window: UserAppWindow } = $props();
  const exe = $derived(executableName(window.process.path));
  const sessions = $derived(previewState.sessions);
  const matches = $derived(matchingSessions(window, sessions));
  const profile = $derived(mediaShortcutProfile(previewState.shortcuts, exe));
  const fallback = $derived(profile?.enabled === true);
  let selection = $state<string | null>(null);
  const selected = $derived(selection ?? matches[0]?.umid ?? (fallback ? "shortcut" : ""));
  const session = $derived(sessions.find((session) => session.umid === selected));
  const usingShortcut = $derived(selected === "shortcut" && fallback);
  const visible = $derived(matches.length > 0 || fallback || knownPlayers.has(exe) || browsers.has(exe) && sessions.length > 0);
  const controls = $derived(usingShortcut ? {
    previous: isMediaShortcut(profile!.previous),
    togglePlayPause: isMediaShortcut(profile!.togglePlayPause),
    next: isMediaShortcut(profile!.next),
  } : session?.controls ?? { previous: false, togglePlayPause: false, next: false });

  function send(action: "previous" | "togglePlayPause" | "next") {
    if (usingShortcut) {
      const actions = {
        previous: MediaTransportAction.Previous,
        togglePlayPause: MediaTransportAction.TogglePlayPause,
        next: MediaTransportAction.Next,
      };
      return invoke(SeelenCommand.MediaSendAppShortcut, { hwnd: window.hwnd, exe, action: actions[action] });
    }
    if (!session) return Promise.reject(new Error("Media session is no longer available"));
    const commands = {
      previous: SeelenCommand.MediaPrev,
      togglePlayPause: SeelenCommand.MediaTogglePlayPause,
      next: SeelenCommand.MediaNext,
    } as const;
    return invoke(commands[action], { id: session.umid });
  }
</script>

{#if visible}
  <div class="weg-preview-media">
    {#if sessions.length > 0 || fallback}
      <select data-skin="default" aria-label={$t("media.source")} value={selected} onchange={(e) => selection = e.currentTarget.value}>
        <option value="">{$t("media.choose_source")}</option>
        {#each sessions as source (source.umid)}
          <option value={source.umid}>{source.owner.name} — {source.title || source.umid}</option>
        {/each}
        {#if fallback}<option value="shortcut">{$t("media.global_shortcuts")}</option>{/if}
      </select>
    {/if}
    {#if session || usingShortcut}
      <MediaTransport target={`${window.hwnd}:${selected}`} playing={session?.playing ?? null} {controls} onaction={send} />
      {#if usingShortcut}<span class="weg-preview-media-note">{$t("media.shortcut_state_unknown")}</span>{/if}
    {:else}
      <span class="weg-preview-media-note">{$t(knownPlayers.has(exe) ? "media.configure_shortcuts" : "media.choose_source")}</span>
    {/if}
  </div>
{/if}

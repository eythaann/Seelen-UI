<script>
  import { onMount } from "svelte";
  import { options } from "./options";
  import { setup } from "./setup.svelte";
  import { state } from "./state.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";

  onMount(setup);
</script>

<div class="power-menu">
  <div class="power-menu-user">
    <img class="power-menu-user-profile" src={convertFileSrc(state.user.profilePicturePath)} alt="" />
    <div class="power-menu-user-email">
      {state.user.email}
    </div>
  </div>
  <div class="power-menu-bye-bye">Goodbye {state.user.name}</div>
  <ul class="power-menu-list">
    {#each options as option}
      <li>
        <button on:click={option.action} class="power-menu-item">
          <Icon iconName={option.icon} />
          <span>{option.key}</span>
        </button>
      </li>
    {/each}
  </ul>
  <div class="power-menu-uptime">Uptime: 2 hours 30 minutes</div>
</div>

<style>
  :global(body) {
    background-color: transparent;
    overflow: hidden;
  }
</style>

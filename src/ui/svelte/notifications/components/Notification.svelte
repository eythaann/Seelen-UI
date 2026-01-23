<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import type {
    AppNotification,
    ToastActionActivationType,
    ToastActionsChild,
    ToastBindingChild,
    ToastImage,
  } from "@seelen-ui/lib/types";
  import { WindowsDateFileTimeToDate } from "libs/ui/svelte/utils";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import FileIcon from "libs/ui/svelte/components/Icon/FileIcon.svelte";
  import moment from "moment";

  interface Props {
    notification: AppNotification;
    onClose?: () => void;
  }

  let { notification, onClose }: Props = $props();

  let inputData = $state<Record<string, string>>({});

  const toastContent = $derived(splitToastContent(notification));
  const logoImage = $derived(toastContent.logoImage);
  const heroImage = $derived(toastContent.heroImage);
  const body = $derived(toastContent.body);
  const actions = $derived(toastContent.actions);

  function handleInputChange(key: string, value: string) {
    inputData = { ...inputData, [key]: value };
  }

  async function handleAction(args: string, method: ToastActionActivationType) {
    try {
      switch (method) {
        case "Protocol":
          await invoke(SeelenCommand.OpenFile, { path: args });
          break;
        default:
          await invoke(SeelenCommand.ActivateNotification, {
            umid: notification.appUmid,
            args,
            inputData,
          });
      }
    } catch (error) {
      console.error("Failed to handle notification action:", error);
    }
  }

  async function handleClose(e: MouseEvent) {
    e.stopPropagation();
    try {
      await invoke(SeelenCommand.NotificationsClose, { id: notification.id });
      onClose?.();
    } catch (error) {
      console.error("Failed to close notification:", error);
    }
  }

  function handleNotificationClick() {
    if (notification.content["@launch"]) {
      handleAction(notification.content["@launch"], notification.content["@activationType"]);
    }
  }

  function splitToastContent(notification: AppNotification) {
    const toast = notification.content;
    const template = toast.visual.binding["@template"];
    const actions = toast.actions?.$value || [];

    let logoImage: ToastImage | null = null;
    let heroImage: ToastImage | null = null;
    const body: ToastBindingChild[] = [];

    for (const entry of toast.visual.binding.$value) {
      if ("image" in entry) {
        if (
          entry.image["@placement"] === "AppLogoOverride" ||
          (!entry.image["@placement"] && !logoImage && template.startsWith("ToastImageAndText"))
        ) {
          logoImage = entry.image;
          continue;
        }

        if (entry.image["@placement"] === "Hero") {
          heroImage = entry.image;
          continue;
        }
      }
      body.push(entry);
    }

    return { logoImage, heroImage, body, actions };
  }
</script>

<div
  class="notification"
  role="button"
  tabindex="0"
  onclick={handleNotificationClick}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      handleNotificationClick();
    }
  }}
>
  <div class="notification-header">
    <div class="notification-header-info">
      <FileIcon class="notification-icon" umid={notification.appUmid} />
      <div>{notification.appName}</div>
      <span>-</span>
      <div>
        {moment(WindowsDateFileTimeToDate(notification.date)).fromNow()}
      </div>
    </div>
    <button data-skin="transparent" onclick={handleClose}>
      <Icon iconName="IoClose" />
    </button>
  </div>

  <div class="notification-body">
    {#if logoImage && logoImage["@src"]}
      <img
        src={logoImage["@src"]}
        alt={logoImage["@alt"] || ""}
        class="notification-body-logo-image"
        class:notification-body-logo-image-circle={logoImage["@hint-crop"] === "Circle"}
      />
    {/if}

    <div class="notification-body-content">
      {#each body as entry, index (index)}
        {#if "text" in entry}
          <p>{entry.text.$value}</p>
        {:else if "image" in entry && entry.image["@src"]}
          {#if entry.image["@placement"] !== "AppLogoOverride" && entry.image["@placement"] !== "Hero"}
            <img src={entry.image["@src"]} alt={entry.image["@alt"] || ""} />
          {/if}
        {:else if "group" in entry}
          <div class="notification-group">
            {#each entry.group.subgroup as subgroup, subIndex (subIndex)}
              <div class="notification-subgroup">
                {#each subgroup.$value as subEntry, subEntryIndex (subEntryIndex)}
                  {#if "text" in subEntry}
                    <p>{subEntry.text.$value}</p>
                  {:else if "image" in subEntry && subEntry.image["@src"]}
                    <img src={subEntry.image["@src"]} alt={subEntry.image["@alt"] || ""} />
                  {/if}
                {/each}
              </div>
            {/each}
          </div>
        {/if}
      {/each}
    </div>

    {#if heroImage && heroImage["@src"]}
      <img
        src={heroImage["@src"]}
        alt={heroImage["@alt"] || ""}
        class="notification-body-hero-image"
      />
    {/if}
  </div>

  {#if actions.length > 0}
    <div class="notification-actions">
      {#each actions as entry, index (index)}
        {#if "input" in entry}
          {@const input = entry.input}
          {#if input["@type"] === "Text"}
            <input
              type="text"
              data-skin="default"
              placeholder={input["@placeHolderContent"] || ""}
              value={inputData[input["@id"]] || ""}
              onclick={(e) => e.stopPropagation()}
              oninput={(e) => handleInputChange(input["@id"], e.currentTarget.value)}
            />
          {:else if input["@type"] === "Selection"}
            <select
              data-skin="default"
              value={inputData[input["@id"]]}
              onclick={(e) => e.stopPropagation()}
              onchange={(e) => handleInputChange(input["@id"], e.currentTarget.value)}
              placeholder={input["@placeHolderContent"]}
            >
              {#each input.selection as opt}
                <option value={opt["@id"]}>{opt["@content"]}</option>
              {/each}
            </select>
          {/if}
        {:else if "action" in entry}
          {@const action = entry.action}
          {#if action["@placement"] !== "ContextMenu"}
            <button
              data-skin="default"
              title={action["@hint-toolTip"] || ""}
              onclick={(e) => {
                e.stopPropagation();
                handleAction(action["@arguments"], action["@activationType"]);
              }}
            >
              {action["@content"]}
            </button>
          {/if}
        {/if}
      {/each}
    </div>
  {/if}
</div>

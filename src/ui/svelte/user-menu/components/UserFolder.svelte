<script lang="ts">
  import type { FolderType } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { path } from "@tauri-apps/api";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "../i18n";
  import { globalState, type FolderData } from "../state.svelte";
  import FilePreview from "./FilePreview.svelte";
  import EmptyList from "./EmptyList.svelte";

  interface Props {
    folder: FolderData;
  }

  let { folder }: Props = $props();

  let showCount = $state(5);

  const isOpen = $derived(globalState.openCategory === folder.type);

  // Filter out .lnk files for Recent folder
  const files = $derived(
    folder.type === "Recent"
      ? folder.files.filter((item) => !item.path.toLowerCase().endsWith(".lnk"))
      : folder.files,
  );

  // Reset showCount when folder closes
  $effect(() => {
    if (!isOpen) {
      showCount = 5;
    }
  });

  async function getPathByFolderType(folderType: FolderType): Promise<string> {
    switch (folderType) {
      case "Recent":
        return (await path.dataDir()) + "\\Microsoft\\Windows\\Recent";
      case "Desktop":
        return await path.desktopDir();
      case "Documents":
        return await path.documentDir();
      case "Downloads":
        return await path.downloadDir();
      case "Music":
        return await path.audioDir();
      case "Pictures":
        return await path.pictureDir();
      case "Videos":
        return await path.videoDir();
      case "Unknown":
      default:
        return await path.homeDir();
    }
  }

  async function openOnExplorer() {
    invoke(SeelenCommand.OpenFile, {
      path: await getPathByFolderType(folder.type),
    });
  }

  function onClickChevron(e: MouseEvent) {
    e.stopPropagation();
    globalState.openCategory = isOpen ? null : folder.type;
  }

  function toggleShowMore(e: MouseEvent) {
    e.stopPropagation();
    showCount = files.length > showCount ? showCount * 2 : 5;
  }

  function getFolderTranslationKey(type: FolderType): string {
    return `folders.${type.toLowerCase()}`;
  }
</script>

<details class="user-directory" open={isOpen}>
  <summary class="user-directory-summary" onclick={openOnExplorer}>
    <Icon iconName={folder.icon as any} class="user-directory-icon" />
    <span class="user-directory-title">{$t(getFolderTranslationKey(folder.type))}</span>
    <button data-skin="transparent" onclick={onClickChevron}>
      <Icon
        iconName="IoIosArrowDown"
        class={[
          "chevron",
          {
            "chevron-active": isOpen,
          },
        ]}
      />
    </button>
  </summary>

  <div class="file-list">
    {#if files.length === 0}
      <EmptyList />
    {/if}

    {#each files.slice(0, showCount) as file (file.path)}
      <FilePreview {file} />
    {/each}

    {#if files.length > 5}
      <button class="user-list-extender" onclick={toggleShowMore}>
        {files.length > showCount ? $t("folders.more_items") : $t("folders.reduce_items")}
      </button>
    {/if}
  </div>
</details>

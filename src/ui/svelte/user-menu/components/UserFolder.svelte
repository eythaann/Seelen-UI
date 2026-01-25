<script lang="ts">
  import { FolderType } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { path } from "@tauri-apps/api";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "../i18n";
  import { globalState } from "../state/mod.svelte";
  import FilePreview from "./FilePreview.svelte";
  import EmptyList from "./EmptyList.svelte";

  interface Props {
    type: FolderType;
    icon: string;
    content: { path: string; displayName: string }[];
  }

  let { type, icon, content }: Props = $props();

  const isOpen = $derived(globalState.openCategory === type);

  async function getPathByFolderType(folderType: FolderType): Promise<string> {
    switch (folderType) {
      case FolderType.Recent:
        return (await path.dataDir()) + "\\Microsoft\\Windows\\Recent";
      case FolderType.Desktop:
        return await path.desktopDir();
      case FolderType.Documents:
        return await path.documentDir();
      case FolderType.Downloads:
        return await path.downloadDir();
      case FolderType.Music:
        return await path.audioDir();
      case FolderType.Pictures:
        return await path.pictureDir();
      case FolderType.Videos:
        return await path.videoDir();
    }
  }

  async function openOnExplorer() {
    invoke(SeelenCommand.OpenFile, {
      path: await getPathByFolderType(type),
    });
  }

  function onClickChevron(e: MouseEvent) {
    e.stopPropagation();
    globalState.openCategory = isOpen ? null : type;
  }

  function getFolderTranslationKey(type: FolderType): string {
    return `folders.${type.toLowerCase()}`;
  }
</script>

<details class="user-directory" open={isOpen}>
  <summary class="user-directory-summary" onclick={openOnExplorer}>
    <Icon iconName={icon as any} class="user-directory-icon" />
    <span class="user-directory-title">{$t(getFolderTranslationKey(type))}</span>
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
    {#if content.length === 0}
      <EmptyList />
    {/if}

    {#each content as file (file.path)}
      <FilePreview path={file.path} displayName={file.displayName} />
    {/each}
  </div>
</details>

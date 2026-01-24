import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import { type File, type FolderChangedArgs, FolderType, type User } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { writable } from "svelte/store";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

const widget = Widget.getCurrent();
const webview = widget.webview;

const settings = writable(await Settings.getAsync());
Settings.onChange((s) => settings.set(s));
settings.subscribe((settings) => {
  locale.set(settings.language || "en");
});

// User state
const user = lazyRune(() => invoke(SeelenCommand.GetUser));
await subscribe(SeelenEvent.UserChanged, user.setByPayload);
await user.init();

// Folder states
const recentFolder = lazyRune(() => invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Recent }));
const desktopFolder = lazyRune(() => invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Desktop }));
const documentsFolder = lazyRune(() =>
  invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Documents })
);
const downloadsFolder = lazyRune(() =>
  invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Downloads })
);
const musicFolder = lazyRune(() => invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Music }));
const picturesFolder = lazyRune(() => invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Pictures }));
const videosFolder = lazyRune(() => invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Videos }));

// Subscribe to folder changes
await subscribe(SeelenEvent.UserFolderChanged, (e: { payload: FolderChangedArgs }) => {
  const { ofFolder, content } = e.payload;
  if (content) {
    switch (ofFolder) {
      case "Recent":
        recentFolder.value = content;
        break;
      case "Desktop":
        desktopFolder.value = content;
        break;
      case "Documents":
        documentsFolder.value = content;
        break;
      case "Downloads":
        downloadsFolder.value = content;
        break;
      case "Music":
        musicFolder.value = content;
        break;
      case "Pictures":
        picturesFolder.value = content;
        break;
      case "Videos":
        videosFolder.value = content;
        break;
    }
  }
});

// Initialize all folders
await Promise.all([
  recentFolder.init(),
  desktopFolder.init(),
  documentsFolder.init(),
  downloadsFolder.init(),
  musicFolder.init(),
  picturesFolder.init(),
  videosFolder.init(),
]);

// Local state
let openCategory = $state<FolderType | null>(null);
webview.onFocusChanged((e) => {
  if (!e.payload) {
    openCategory = null;
  }
});

export interface FolderData {
  type: FolderType;
  icon: string;
  files: File[];
}

class State {
  get user(): User {
    return user.value;
  }

  get openCategory(): FolderType | null {
    return openCategory;
  }
  set openCategory(value: FolderType | null) {
    openCategory = value;
  }

  get folders(): FolderData[] {
    return [
      { type: FolderType.Recent, icon: "MdOutlineHistory", files: recentFolder.value },
      { type: FolderType.Desktop, icon: "HiOutlineDesktopComputer", files: desktopFolder.value },
      { type: FolderType.Documents, icon: "IoDocumentsOutline", files: documentsFolder.value },
      { type: FolderType.Downloads, icon: "PiDownloadSimpleBold", files: downloadsFolder.value },
      { type: FolderType.Music, icon: "BsFileEarmarkMusic", files: musicFolder.value },
      { type: FolderType.Pictures, icon: "IoImageOutline", files: picturesFolder.value },
      { type: FolderType.Videos, icon: "PiVideo", files: videosFolder.value },
    ];
  }
}

export const globalState = new State();

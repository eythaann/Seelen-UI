import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { FolderType } from "@seelen-ui/lib/types";

let desktop = $state(
  await invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Desktop }),
);

let downloads = $state(
  await invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Downloads }),
);

let documents = $state(
  await invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Documents }),
);

let music = $state(
  await invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Music }),
);

let pictures = $state(
  await invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Pictures }),
);

let videos = $state(
  await invoke(SeelenCommand.GetUserFolderContent, { folderType: FolderType.Videos }),
);

subscribe(SeelenEvent.UserFolderChanged, ({ payload: { ofFolder, content } }) => {
  switch (ofFolder) {
    case FolderType.Desktop:
      desktop = content;
      break;
    case FolderType.Downloads:
      downloads = content;
      break;
    case FolderType.Documents:
      documents = content;
      break;
    case FolderType.Music:
      music = content;
      break;
    case FolderType.Pictures:
      pictures = content;
      break;
    case FolderType.Videos:
      videos = content;
      break;
  }
});

function pathAsItem(path: string) {
  return {
    path,
    displayName: path.split(/[\\/]/g).pop() || "",
  };
}

function predicate(path: string): boolean {
  let lowercased = path.toLowerCase();
  return !lowercased.endsWith(".ini") && !lowercased.endsWith(".tmp");
}

const _knownFolders: Record<FolderType, FolderData> = $derived.by(() => {
  return {
    [FolderType.Recent]: {
      icon: "MdOutlineHistory",
      content: [],
    },
    [FolderType.Desktop]: {
      icon: "HiOutlineDesktopComputer",
      content: desktop.filter(predicate).map(pathAsItem),
    },
    [FolderType.Downloads]: {
      icon: "PiDownloadSimpleBold",
      content: downloads.filter(predicate).map(pathAsItem),
    },
    [FolderType.Documents]: {
      icon: "IoDocumentsOutline",
      content: documents.filter(predicate).map(pathAsItem),
    },
    [FolderType.Music]: {
      icon: "BsFileEarmarkMusic",
      content: music.filter(predicate).map(pathAsItem),
    },
    [FolderType.Pictures]: {
      icon: "IoImageOutline",
      content: pictures.filter(predicate).map(pathAsItem),
    },
    [FolderType.Videos]: {
      icon: "PiVideo",
      content: videos.filter(predicate).map(pathAsItem),
    },
  };
});

export interface FolderData {
  icon: string;
  content: { path: string; displayName: string }[];
}

export const knownFolders = {
  get value() {
    return _knownFolders;
  },
};

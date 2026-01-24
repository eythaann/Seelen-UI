import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { FolderType, type StartMenuItem } from "@seelen-ui/lib/types";

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

function pathAsItem(path: string): StartMenuItem {
  return {
    path,
    umid: null,
    display_name: path.split(/[\\/]/g).pop() || "",
    target: null,
    toast_activator: null,
  };
}

function predicate(path: string): boolean {
  let lowercased = path.toLowerCase();
  return !lowercased.endsWith(".ini") && !lowercased.endsWith(".tmp");
}

const _foldersAsStartMenuItems = $derived.by(() => {
  return [
    desktop.filter(predicate).map(pathAsItem),
    downloads.filter(predicate).map(pathAsItem),
    documents.filter(predicate).map(pathAsItem),
    music.filter(predicate).map(pathAsItem),
    pictures.filter(predicate).map(pathAsItem),
    videos.filter(predicate).map(pathAsItem),
  ].flat();
});

export const foldersAsStartMenuItems = {
  get value() {
    return _foldersAsStartMenuItems;
  },
};

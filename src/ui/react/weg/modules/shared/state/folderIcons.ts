import { signal } from "@preact/signals";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { emit, listen } from "@tauri-apps/api/event";
import { debounce } from "lodash";

/**
 * Custom folder icons, stored as base64 data URLs keyed by folder id.
 *
 * The chosen image is imported into Seelen's own storage (read into a data URL
 * and written to a JSON file inside the weg widget data dir) instead of
 * referencing the original file path, so the icon survives even if the source
 * file is moved or deleted.
 */
type FolderIconMap = Record<string, string>;

const FILENAME = "folder-icons.json";
const SYNC_EVENT = "hidden::sync-folder-icons";
const CLIENT_ID = crypto.randomUUID();

interface SyncPayload {
  source: string;
  icons: FolderIconMap;
}

async function loadInitial(): Promise<FolderIconMap> {
  try {
    const raw = await invoke(SeelenCommand.ReadFile, { filename: FILENAME });
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed as FolderIconMap : {};
  } catch {
    return {};
  }
}

export const $folder_icons = signal<FolderIconMap>(await loadInitial());

const persist = debounce((icons: FolderIconMap) => {
  invoke(SeelenCommand.WriteFile, {
    filename: FILENAME,
    content: JSON.stringify(icons),
  }).catch(() => {});
}, 300);

listen<SyncPayload>(SYNC_EVENT, ({ payload }) => {
  if (payload.source === CLIENT_ID) return;
  $folder_icons.value = payload.icons;
});

function commit(next: FolderIconMap) {
  $folder_icons.value = next;
  persist(next);
  emit<SyncPayload>(SYNC_EVENT, { source: CLIENT_ID, icons: next }).catch(() => {});
}

export const $folder_icon_actions = {
  set(folderId: string, dataUrl: string) {
    commit({ ...$folder_icons.value, [folderId]: dataUrl });
  },
  remove(folderId: string) {
    if (!(folderId in $folder_icons.value)) return;
    const next = { ...$folder_icons.value };
    delete next[folderId];
    commit(next);
  },
};

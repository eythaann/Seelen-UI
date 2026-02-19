import { DragDropProvider, DragOverlay } from "@dnd-kit/react";
import { move } from "@dnd-kit/helpers";
import { WegItemType } from "@seelen-ui/lib/types";
import { useTranslation } from "react-i18next";

import { FileOrFolder } from "../item/infra/File.tsx";
import { MediaSession } from "../item/infra/MediaSession.tsx";
import { Separator } from "../item/infra/Separator.tsx";
import { StartMenu } from "../item/infra/StartMenu.tsx";
import { UserApplication } from "../item/infra/UserApplication.tsx";

import type { SwItem } from "../shared/types.ts";

import { $dock_state } from "../shared/state/items.ts";
import { DraggableItem } from "./DraggableItem.tsx";
import { ShowDesktopModule } from "../item/infra/ShowDesktop.tsx";

export function DockItems() {
  const { t } = useTranslation();

  const isEmpty = $dock_state.value.items.filter((c) => c.type !== WegItemType.Separator).length === 0;

  return (
    <DragDropProvider
      onDragOver={(event) => {
        const newItems = move($dock_state.value.items, event);
        $dock_state.value = { ...$dock_state.value, items: newItems };
      }}
    >
      <div className="weg-items">
        {isEmpty ? <span className="weg-empty-state-label">{t("weg.empty")}</span> : (
          $dock_state.value.items.map((item, index) => (
            <DraggableItem item={item} key={item.id} index={index}>
              {ItemByType(item, false)}
            </DraggableItem>
          ))
        )}
      </div>

      <DragOverlay>
        {(source) => {
          const item = $dock_state.value.items.find((c) => c.id === source.id);
          return item ? ItemByType(item, true) : null;
        }}
      </DragOverlay>
    </DragDropProvider>
  );
}

function ItemByType(item: SwItem, isOverlay: boolean) {
  if (item.type === WegItemType.Pinned) {
    if (item.subtype === "App") {
      return <UserApplication key={item.id} item={item} isOverlay={isOverlay} />;
    }
    return <FileOrFolder key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Temporal) {
    return <UserApplication key={item.id} item={item} isOverlay={isOverlay} />;
  }

  if (item.type === WegItemType.StartMenu) {
    return <StartMenu key={item.id} item={item} />;
  }

  if (item.type === WegItemType.ShowDesktop) {
    return <ShowDesktopModule key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Media) {
    return <MediaSession key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Separator) {
    return <Separator key={item.id} item={item} />;
  }

  return null;
}

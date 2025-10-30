import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { type File, FolderType } from "@seelen-ui/lib/types";
import { AnimatedPopover } from "@shared/components/AnimatedWrappers";
import { Icon } from "@shared/components/Icon";
import type { IconName } from "@shared/components/Icon/icons";
import { useWindowFocusChange } from "@shared/hooks";
import { path } from "@tauri-apps/api";
import { t } from "i18next";
import type { VNode } from "preact";
import { useEffect, useState } from "react";
import { useSelector } from "react-redux";

import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { Selectors } from "../../shared/store/app.ts";

import { UserFolder } from "./UserFolder.tsx";
import { UserProfile } from "./UserProfile.tsx";

function folderTypeToIcon(
  folderType: FolderType,
): { icon: IconName; category: FolderType } {
  switch (folderType) {
    case "Recent": {
      return { category: folderType, icon: "MdOutlineHistory" };
    }
    case "Desktop": {
      return { category: folderType, icon: "HiOutlineDesktopComputer" };
    }
    case "Documents": {
      return { category: folderType, icon: "IoDocumentsOutline" };
    }
    case "Downloads": {
      return { category: folderType, icon: "PiDownloadSimpleBold" };
    }
    case "Pictures": {
      return { category: folderType, icon: "IoImageOutline" };
    }
    case "Videos": {
      return { category: folderType, icon: "PiVideo" };
    }
    case "Music": {
      return { category: folderType, icon: "BsFileEarmarkMusic" };
    }
    default: {
      throw new Error(`The given parameter: ${folderType} incovertible`);
    }
  }
}

export interface UserHomeFolder {
  category: FolderType;
  content: File[];
  icon: IconName;
}

function UserHome() {
  const [categoryOpen, setCategoryOpen] = useState<FolderType>(FolderType.Unknown);

  const user = useSelector(Selectors.user);
  const folders: UserHomeFolder[] = [
    {
      ...folderTypeToIcon(FolderType.Recent),
      content: useSelector(Selectors.userRecentFolder),
    },
    {
      ...folderTypeToIcon(FolderType.Desktop),
      content: useSelector(Selectors.userDesktopFolder),
    },
    {
      ...folderTypeToIcon(FolderType.Documents),
      content: useSelector(Selectors.userDocumentsFolder),
    },
    {
      ...folderTypeToIcon(FolderType.Downloads),
      content: useSelector(Selectors.userDownloadsFolder),
    },
    {
      ...folderTypeToIcon(FolderType.Music),
      content: useSelector(Selectors.userMusicFolder),
    },
    {
      ...folderTypeToIcon(FolderType.Pictures),
      content: useSelector(Selectors.userPicturesFolder),
    },
    {
      ...folderTypeToIcon(FolderType.Videos),
      content: useSelector(Selectors.userVideosFolder),
    },
  ];

  return (
    <BackgroundByLayersV2
      prefix="userhome"
      className="userhome"
      onContextMenu={(e) => e.stopPropagation()}
    >
      {user && <UserProfile user={user} />}

      <hr />
      <span className="userhome-label">{t("userhome.folders.title")}</span>
      {folders.map((item) => (
        <UserFolder
          key={item.category}
          folderProps={item}
          categoryOpen={categoryOpen}
          setCategoryOpen={setCategoryOpen}
        />
      ))}

      <hr />
      <span className="userhome-label">
        {t("userhome.seelen_options.title")}
      </span>
      <ul className="userhome-seelen-options">
        <li
          className="userhome-seelen-option-item"
          onClick={async () => invoke(SeelenCommand.OpenFile, { path: await path.appDataDir() })}
        >
          <Icon iconName="TbFolderCog" />
          <span className="userhome-seelen-option-item-title">
            {t("userhome.seelen_options.open_installation_folder")}
          </span>
        </li>
        <li
          className="userhome-seelen-option-item"
          onClick={async () => invoke(SeelenCommand.OpenFile, { path: await path.appLogDir() })}
        >
          <Icon iconName="TbLogs" />
          <span className="userhome-seelen-option-item-title">
            {t("userhome.seelen_options.open_log_folder")}
          </span>
        </li>
      </ul>
    </BackgroundByLayersV2>
  );
}

export interface UserHomeModuleProps {
  setOpen: (open: boolean) => void;
  children: VNode;
}

export function WithUserHome({ setOpen, children }: UserHomeModuleProps) {
  const [openPreview, setOpenPreview] = useState(false);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  useEffect(() => setOpen(openPreview), [openPreview]);

  return (
    <AnimatedPopover
      animationDescription={{
        openAnimationName: "userhome-open",
        closeAnimationName: "userhome-close",
      }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      content={<UserHome />}
      // destroyTooltipOnHide
    >
      {children}
    </AnimatedPopover>
  );
}

import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { File, FolderType } from '@seelen-ui/lib/types';
import { path } from '@tauri-apps/api';
import { Tooltip } from 'antd';
import { t } from 'i18next';
import { PropsWithChildren, useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';
import { AnimatedPopover } from 'src/apps/shared/components/AnimatedWrappers';

import { Icon } from '../../../../shared/components/Icon';
import { useWindowFocusChange } from '../../../../shared/hooks';
import { UserAppHistory } from './UserAppHistory';
import { UserFolder } from './UserFolder';
import { UserProfile } from './UserProfile';

interface UserHomeProps {}

function folderTypeToIcon(folderType: FolderType): { icon: string; category: FolderType } {
  switch (folderType) {
    case 'Recent': {
      return { category: folderType, icon: 'MdOutlineHistory' };
    }
    case 'Desktop': {
      return { category: folderType, icon: 'HiOutlineDesktopComputer' };
    }
    case 'Documents': {
      return { category: folderType, icon: 'IoDocumentsOutline' };
    }
    case 'Downloads': {
      return { category: folderType, icon: 'PiDownloadSimpleBold' };
    }
    case 'Pictures': {
      return { category: folderType, icon: 'IoImageOutline' };
    }
    case 'Videos': {
      return { category: folderType, icon: 'PiVideo' };
    }
    case 'Music': {
      return { category: folderType, icon: 'BsFileEarmarkMusic' };
    }
    default: {
      throw new Error(`The given parameter: ${folderType} incovertible`);
    }
  }
}

export interface UserHomeFolder {
  category: FolderType;
  content: File[];
  icon: string;
}

function UserHome({}: UserHomeProps) {
  const [categoryOpen, setCategoryOpen] = useState<FolderType>('Unknown');

  const user = useSelector(Selectors.user);
  const folders: UserHomeFolder[] = [
    { ...folderTypeToIcon('Recent'), content: useSelector(Selectors.userRecentFolder) },
    { ...folderTypeToIcon('Desktop'), content: useSelector(Selectors.userDesktopFolder) },
    { ...folderTypeToIcon('Documents'), content: useSelector(Selectors.userDocumentsFolder) },
    { ...folderTypeToIcon('Downloads'), content: useSelector(Selectors.userDownloadsFolder) },
    { ...folderTypeToIcon('Music'), content: useSelector(Selectors.userMusicFolder) },
    { ...folderTypeToIcon('Pictures'), content: useSelector(Selectors.userPicturesFolder) },
    { ...folderTypeToIcon('Videos'), content: useSelector(Selectors.userVideosFolder) },
  ];

  return (
    <BackgroundByLayersV2
      prefix="userhome"
      className="userhome"
      onContextMenu={(e) => e.stopPropagation()}
    >
      {user && <UserProfile user={user} />}

      <hr />
      <UserAppHistory />

      <hr />
      <span className="userhome-label">{t('userhome.folders.title')}</span>
      {folders.map((item) => (
        <UserFolder
          key={item.category}
          folderProps={item}
          categoryOpen={categoryOpen}
          setCategoryOpen={setCategoryOpen}
        />
      ))}

      <hr />
      <span className="userhome-label">{t('userhome.seelen_options.title')}</span>
      <ul className="userhome-seelen-options">
        <li
          className="userhome-seelen-option-item"
          onClick={async () => invoke(SeelenCommand.OpenFile, { path: await path.appDataDir() })}
        >
          <Icon iconName="MdOutlineInstallDesktop" />
          <span className="userhome-seelen-option-item-title">
            {t('userhome.seelen_options.open_installation_folder')}
          </span>
        </li>
        <li
          className="userhome-seelen-option-item"
          onClick={async () => invoke(SeelenCommand.OpenFile, { path: await path.appLogDir() })}
        >
          <Icon iconName="TbLogs" />
          <span className="userhome-seelen-option-item-title">
            {t('userhome.seelen_options.open_log_folder')}
          </span>
        </li>
      </ul>

      <hr />
      <span className="userhome-label">{t('settings.power')}</span>
      <div className="userhome-power">
        <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.lock')}>
          <button className="userhome-power-button" onClick={() => invoke(SeelenCommand.Lock)}>
            <Icon iconName="BiLock" />
          </button>
        </Tooltip>
        <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.sleep')}>
          <button className="userhome-power-button" onClick={() => invoke(SeelenCommand.Suspend)}>
            <Icon iconName="BiMoon" />
          </button>
        </Tooltip>
        <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.restart')}>
          <button className="userhome-power-button" onClick={() => invoke(SeelenCommand.Restart)}>
            <Icon iconName="VscDebugRestart" />
          </button>
        </Tooltip>
        <Tooltip mouseLeaveDelay={0} arrow={false} title={t('settings.shutdown')}>
          <button className="userhome-power-button" onClick={() => invoke(SeelenCommand.Shutdown)}>
            <Icon iconName="GrPower" />
          </button>
        </Tooltip>
      </div>
    </BackgroundByLayersV2>
  );
}

export interface UserHomeModuleProps extends PropsWithChildren {
  setOpen: (open: boolean) => void;
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
        maxAnimationTimeMs: 500,
        openAnimationName: 'userhome-open',
        closeAnimationName: 'userhome-close',
      }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={<UserHome />}
      // destroyTooltipOnHide
    >
      {children}
    </AnimatedPopover>
  );
}

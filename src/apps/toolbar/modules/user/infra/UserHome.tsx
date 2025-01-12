import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { File, FolderType, User } from '@seelen-ui/lib/types';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Popover, Tooltip } from 'antd';
import { t } from 'i18next';
import { PropsWithChildren, useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';

import { AppHistoryItem } from '../../shared/store/domain';

import { Icon } from '../../../../shared/components/Icon';
import { useWindowFocusChange } from '../../../../shared/hooks';
import { UserFolder } from './UserFolder';

interface UserHomeProps {
}

function folderTypeToIcon(folderType: FolderType): { icon: string; category: FolderType } {
  switch (folderType) {
    case 'Recent': {
      return { category: folderType, icon: 'MdOutlineHistory' }; // GoHistory
    }
    case 'Documents': {
      return { category: folderType, icon: 'IoDocumentsSharp' };
    }
    case 'Downloads': {
      return { category: folderType, icon: 'PiDownloadDuotone' }; // RiDownload2Fill, RxDownload
    }
    case 'Pictures': {
      return { category: folderType, icon: 'AiTwotonePicture' };
    }
    case 'Videos': {
      return { category: folderType, icon: 'PiVideo' };
    }
    case 'Music': {
      return { category: folderType, icon: 'RiFileMusicFill' };
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

function UserHome({ }: UserHomeProps) {
  const [historyCount, setHistoryCount] = useState(5);
  const [categoryOpen, setCategoryOpen] = useState<FolderType>('Recent');

  const user: User = useSelector(Selectors.user);
  const folders: UserHomeFolder[] = [
    { ...folderTypeToIcon('Recent'), content: useSelector(Selectors.userRecentFolder) },
    { ...folderTypeToIcon('Documents'), content: useSelector(Selectors.userDocumentsFolder) },
    { ...folderTypeToIcon('Downloads'), content: useSelector(Selectors.userDownloadsFolder) },
    { ...folderTypeToIcon('Pictures'), content: useSelector(Selectors.userPicturesFolder) },
    { ...folderTypeToIcon('Videos'), content: useSelector(Selectors.userVideosFolder) },
    { ...folderTypeToIcon('Music'), content: useSelector(Selectors.userMusicFolder) },
  ];
  const history: AppHistoryItem[] = useSelector(Selectors.history);

  return (
    <BackgroundByLayersV2 prefix="userhome" className="userhome" onContextMenu={(e) => e.stopPropagation()}>
      <div className="userhome-profile-container">
        <div className="userhome-profile-picture-container">
          <div className="userhome-profile-picture">
            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={t('userhome.profile.accounts')}
              placement="right"
            >
              <img
                className="userhome-profile-picture-img"
                src={convertFileSrc(user.profilePicturePath ?? LAZY_CONSTANTS.MISSING_ICON_PATH)}
                onClick={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:accounts' })}
              />
            </Tooltip>
            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={t('userhome.profile.log-out')}
              placement="right"
            >
              <button className="userhome-profile-button-signout" onClick={() => invoke(SeelenCommand.LogOut)}>
                <Icon iconName="BiLogOut" />
              </button>
            </Tooltip>
            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={t('userhome.profile.lock')}
              placement="right"
            >
              <button className="userhome-profile-button-lock" onClick={() => invoke(SeelenCommand.Lock)}>
                <Icon iconName="BiLock" />
              </button>
            </Tooltip>
          </div>
        </div>
        <div className="userhome-profile-actions">
          <Tooltip
            mouseLeaveDelay={0}
            arrow={false}
            title={t('placeholder.open_user_folder')}
            placement="right">
            <button
              className="userhome-profile-username"
              onClick={() => invoke(SeelenCommand.OpenFile, { path: user.profileHomePath })}
            >
              <Icon iconName="RiFolderUserFill" />
              <div className="userhome-profile-username-text">{user.domain + '\\' + user.name}</div>
            </button>
          </Tooltip>
          <Tooltip
            mouseLeaveDelay={0}
            arrow={false}
            title={t('placeholder.open_mails')}
            placement="right">
            <div
              className="userhome-profile-mails"
            >
              <Icon iconName="TbMail" />
              <div className="userhome-profile-mails-text">{user.email}</div>
            </div>
          </Tooltip>
          <Tooltip
            mouseLeaveDelay={0}
            arrow={false}
            title={t('userhome.profile.open-onedrive')}
            placement="right">
            <button
              className="userhome-profile-onedrive"
              onClick={() => invoke(SeelenCommand.OpenFile, { path: user.oneDrivePath })}
            >
              <Icon iconName="ImOnedrive" />
              <div className="userhome-profile-onedrive-text">{t('userhome.profile.one-drive')}</div>
            </button>
          </Tooltip>
          <Tooltip
            mouseLeaveDelay={0}
            arrow={false}
            title={t('userhome.profile.passwords')}
            placement="right">
            <button
              className="userhome-profile-passwords"
              onClick={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:signinoptions' })}
            >
              <Icon iconName="RiLockPasswordFill" />
              <div className="userhome-profile-passwords-text">****</div>
            </button>
          </Tooltip>
        </div>
      </div>
      <ul className="userhome-folders">
        {folders.map((item) => <UserFolder key={item.category} folderProps={item} categoryOpen={categoryOpen} setCategoryOpen={setCategoryOpen}/>)}
      </ul>
      { history && history.length != 0 &&
        <>
          <div className="userhome-title">{t('userhome.history.title')}</div>
          <ul className="userhome-history">
            {history.slice(0, historyCount).map((item, index) => (
              <Tooltip
                key={index}
                mouseLeaveDelay={0}
                arrow={false}
                title={item.name + ' - ' + item.title}
                placement="right">
                <li className="userhome-history-item" onClick={() => invoke(SeelenCommand.RequestFocus, { hwnd: item.hwnd })}>
                  <img className="userhome-history-item-icon" src={convertFileSrc(item.iconPath)} />
                  <div className="userhome-history-item-title">{item.name} - {item.title}</div>
                  <div className="userhome-history-item-date" >{item.date.fromNow()}</div>
                </li>
              </Tooltip>
            ))}
          </ul>
          { history.length > 5 &&
            <button className="userhome-folder-history-extender" onClick={() => setHistoryCount(history.length > historyCount ? historyCount * 2 : 5)}>{history.length > historyCount ? t('userhome.history.more-items') : t('userhome.history.reduce-items')}</button>
          }
        </>
      }
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
    <Popover
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={<UserHome />}
      destroyTooltipOnHide
    >
      {children}
    </Popover>
  );
}
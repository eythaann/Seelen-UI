import { invoke, SeelenCommand, Settings, ThemeList } from '@seelen-ui/lib';
import { ApplicationHistoryEntry, File, FolderType, Theme, User } from '@seelen-ui/lib/types';
import { path } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import { t } from 'i18next';
import moment from 'moment';
import { PropsWithChildren, useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';
import { AnimatedPopover } from 'src/apps/shared/components/AnimatedWrappers';
import { cx } from 'src/apps/shared/styles';

import { Icon } from '../../../../shared/components/Icon';
import { useIcon, useWindowFocusChange } from '../../../../shared/hooks';
import { ThemeTool } from './ThemeTool';
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

type SettingsType = 'Theme' | undefined;

export interface UserHomeFolder {
  category: FolderType;
  content: File[];
  icon: string;
}

const filter = (history: ApplicationHistoryEntry[]) =>
  history
    // Do not want to show the Seelen items
    .filter((item) => !item.isSeelen)
    // Filter those which are there bacause the Seelen are filtered out
    .filter((item, index, array) => index == 0 || array[index - 1]?.application.hwnd != item.application.hwnd)
    // Only show the last 20 of the history max!
    .slice(0, 20);

function UserHome({ }: UserHomeProps) {
  const [historyOpen, setHistoryOpen] = useState(false);
  const [historyIsLocal, setHistoryIsLocal] = useState(false);
  const [historyCount, setHistoryCount] = useState(5);
  const [categoryOpen, setCategoryOpen] = useState<FolderType>('Unknown');
  const [openSettings, setOpenSettings] = useState<SettingsType>(undefined);

  const [themes, setThemes] = useState<Theme[]>([]);
  const [homePath, setHomePath] = useState<string | undefined>(undefined);
  const [logHomePath, setLogHomePath] = useState<string | undefined>(undefined);
  const [selectedThemes, setSelectedThemes] = useState<string[] | undefined>(undefined);
  const [settings, setSettings] = useState<Settings | undefined>(undefined);

  const user: User = useSelector(Selectors.user);
  const folders: UserHomeFolder[] = [
    { ...folderTypeToIcon('Recent'), content: useSelector(Selectors.userRecentFolder) },
    { ...folderTypeToIcon('Documents'), content: useSelector(Selectors.userDocumentsFolder) },
    { ...folderTypeToIcon('Downloads'), content: useSelector(Selectors.userDownloadsFolder) },
    { ...folderTypeToIcon('Pictures'), content: useSelector(Selectors.userPicturesFolder) },
    { ...folderTypeToIcon('Videos'), content: useSelector(Selectors.userVideosFolder) },
    { ...folderTypeToIcon('Music'), content: useSelector(Selectors.userMusicFolder) },
  ];
  const storeHistory: ApplicationHistoryEntry[] = useSelector(Selectors.history);
  const storeOnMonitorHistory: ApplicationHistoryEntry[] = useSelector(Selectors.historyOnMonitor);

  const history: ApplicationHistoryEntry[] = historyIsLocal ? filter(storeOnMonitorHistory) : filter(storeHistory);

  useEffect(() => {
    ThemeList.getAsync().then((values) => setThemes(values.all()));
    Settings.getAsync().then((settings) => {
      setSettings(settings);
      setSelectedThemes(settings.inner.selectedThemes);
    });
    path.homeDir().then((homePath) => {
      path.join(homePath, 'AppData', 'Roaming', 'com.seelen.seelen-ui').then((path) => setHomePath(path));
      path.join(homePath, 'AppData', 'Local', 'com.seelen.seelen-ui').then((path) => setLogHomePath(path));
    });
  }, []);

  useEffect(() => {
    if (settings && settings.inner.selectedThemes.length != selectedThemes?.length) {
      const innerSettings = settings.inner;

      innerSettings.selectedThemes = [ ...selectedThemes! ] ;

      invoke(SeelenCommand.StateWriteSettings, { settings: innerSettings });
    }
  }, [selectedThemes]);

  return (
    <BackgroundByLayersV2 prefix="userhome" className="userhome" onContextMenu={(e) => e.stopPropagation()}>
      <div className="userhome-profile-container">
        <div className="userhome-profile-picture-container">
          <div className="userhome-profile-picture">
            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={t('userhome.profile.accounts')}
              placement="bottom"
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
            title={t('userhome.profile.open_user_folder')}
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
            title={t('userhome.profile.open_onedrive')}
            placement="right">
            <button
              className="userhome-profile-onedrive"
              onClick={() => invoke(SeelenCommand.OpenFile, { path: user.oneDrivePath })}
            >
              <Icon iconName="ImOnedrive" />
              <div className="userhome-profile-onedrive-text">{user.email}</div>
            </button>
          </Tooltip>
        </div>
      </div>
      { history &&
        <>
          <div className="userhome-title" onClick={() => setHistoryOpen(!historyOpen)}>
            <span>{t('userhome.history.title')}</span>
            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={historyIsLocal ? t('userhome.history.local') : t('userhome.history.global')}
              placement="right">
              <Icon
                iconName={historyIsLocal ? 'PiAppWindowLight' : 'AiOutlineGlobal'}
                className={cx('userhome-history-location', { 'userhome-history-location-local': historyIsLocal, 'userhome-history-location-global': !historyIsLocal })}
                onClick={(e) => {
                  setHistoryIsLocal(!historyIsLocal);
                  e.preventDefault();
                  e.stopPropagation();
                }}
              />
            </Tooltip>
            <Icon iconName="IoIosArrowDown" className={cx('userhome-history-expander', { 'userhome-history-expander-open': historyOpen })} />
          </div>
          <ul className={cx('userhome-history', { 'userhome-history-open': historyOpen })}>
            { !historyIsLocal && filter(storeHistory).slice(0, historyCount).map((item, index) => (
              <HistoryItem key={index} item={item} />
            ))}
            { historyIsLocal && filter(storeOnMonitorHistory).slice(0, historyCount).map((item, index) => (
              <HistoryItem key={index} item={item} />
            ))}
            { history.length > 5 &&
              <button className="userhome-folder-history-extender" onClick={() => setHistoryCount(history.length > historyCount ? historyCount * 2 : 5)}>{history.length > historyCount ? t('userhome.history.more_items') : t('userhome.history.reduce_items')}</button>
            }
          </ul>
        </>
      }
      <ul className="userhome-folders">
        <div className="userhome-title">{t('userhome.folders.title')}</div>
        {folders.map((item) => <UserFolder key={item.category} folderProps={item} categoryOpen={categoryOpen} setCategoryOpen={setCategoryOpen}/>)}
      </ul>
      <ul className="userhome-seelen-options">
        <span className="userhome-title">{t('userhome.seelen_options.title')}</span>
        <AnimatedPopover
          animationDescription={{
            maxAnimationTimeMs: 500,
            openAnimationName: 'userhome-quicksettings-open',
            closeAnimationName: 'userhome-quicksettings-close',
          }}
          open={openSettings == 'Theme'}
          trigger="click"
          onOpenChange={(open) => {
            if (!open && openSettings == 'Theme') {
              setOpenSettings(undefined);

              return;
            }

            setOpenSettings(open ? 'Theme' : openSettings);
          }}
          arrow={false}
          placement="right"
          content={
            <BackgroundByLayersV2 prefix="userhome-quicksettings" className="userhome-quicksettings" onContextMenu={(e) => e.stopPropagation()}>
              <ThemeTool dataSource={themes} usingThemes={selectedThemes ? selectedThemes : []} setSelectedThemes={setSelectedThemes} />
            </BackgroundByLayersV2>
          }
          destroyTooltipOnHide
        >
          <li className="userhome-seelen-option-item">
            <Icon iconName="MdStyle" />
            <span className="userhome-seelen-option-item-title">{t('userhome.seelen_options.theme')}</span>
          </li>
        </AnimatedPopover>
        <li className="userhome-seelen-option-item" onClick={() => invoke(SeelenCommand.OpenFile, { path: homePath })}>
          <Icon iconName="MdOutlineInstallDesktop" />
          <span className="userhome-seelen-option-item-title">{t('userhome.seelen_options.open_installation_folder')}</span>
        </li>
        <li className="userhome-seelen-option-item" onClick={() => invoke(SeelenCommand.OpenFile, { path: logHomePath })}>
          <Icon iconName="TbLogs" />
          <span className="userhome-seelen-option-item-title">{t('userhome.seelen_options.open_log_folder')}</span>
        </li>
        <li className="userhome-seelen-option-item" onClick={() => invoke(SeelenCommand.ShowAppSettings)}>
          <Icon iconName="RiSettings3Fill" />
          <span className="userhome-seelen-option-item-title">{t('userhome.seelen_options.settings')}</span>
        </li>
      </ul>
      <div className="userhome-power">
        <span className="userhome-power-label">{t('userhome.power.title')}</span>
        <Tooltip mouseLeaveDelay={0} arrow={false} title={t('userhome.power.log_out')}>
          <button className="userhome-power-button" onClick={() => invoke(SeelenCommand.LogOut)}>
            <Icon iconName="BiLogOut" />
          </button>
        </Tooltip>
        <Tooltip mouseLeaveDelay={0} arrow={false} title={t('userhome.power.sleep')}>
          <button className="userhome-power-button" onClick={() => invoke(SeelenCommand.Suspend)}>
            <Icon iconName="BiMoon" />
          </button>
        </Tooltip>
        <Tooltip mouseLeaveDelay={0} arrow={false} title={t('userhome.power.restart')}>
          <button className="userhome-power-button" onClick={() => invoke(SeelenCommand.Restart)}>
            <Icon iconName="VscDebugRestart" />
          </button>
        </Tooltip>
        <Tooltip mouseLeaveDelay={0} arrow={false} title={t('userhome.power.shutdown')}>
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
      destroyTooltipOnHide
    >
      {children}
    </AnimatedPopover>
  );
}

interface HistoryItemProps {
  item: ApplicationHistoryEntry;
}

function HistoryItem({ item }: HistoryItemProps) {
  const iconSrc = useIcon({ path: item.application.exe, umid: item.application.umid }) || convertFileSrc(LAZY_CONSTANTS.MISSING_ICON_PATH);

  return (
    <Tooltip
      mouseLeaveDelay={0}
      arrow={false}
      title={item.application.name + ' - ' + item.application.title}
      placement="right">
      <li className="userhome-history-item" onClick={() => invoke(SeelenCommand.RequestFocus, { hwnd: item.application.hwnd })}>
        <img className="userhome-history-item-icon" src={iconSrc} />
        <div className="userhome-history-item-title">{item.application.name} - {item.application.title}</div>
        <div className="userhome-history-item-date" >{moment(new Date(Number(item.focusDate))).fromNow()}</div>
      </li>
    </Tooltip>
  );
}
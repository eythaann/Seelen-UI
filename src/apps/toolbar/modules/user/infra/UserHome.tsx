import { invoke, RecentFile, SeelenCommand, User } from '@seelen-ui/lib';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Popover, Tooltip } from 'antd';
import { t } from 'i18next';
import moment from 'moment';
import { PropsWithChildren, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';

import { AppHistoryItem } from '../../shared/store/domain';

import { Icon } from '../../../../shared/components/Icon';
import { useWindowFocusChange } from '../../../../shared/hooks';

interface HomeProps {
}

const EPOCH_DIFF_MILLISECONDS = 11644473600000n;

function WindowsDateFileTimeToDate(fileTime: number) {
  return new Date(Number(BigInt(fileTime) / 10000n - EPOCH_DIFF_MILLISECONDS));
}

function UserHome({ }: HomeProps) {
  const [historyCount, setHistoryCount] = useState(5);
  const [recentCount, setRecentCount] = useState(5);

  const user: User = useSelector(Selectors.user);
  const recentFiles: RecentFile[] = useSelector(Selectors.userRecentFolder);
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
              title={t('userhome.profile.log-out')}
              placement="bottom"
            >
              <button className="userhome-profile-button-signout" onClick={() => invoke(SeelenCommand.LogOut)}>
                <Icon iconName="BiLogOut" />
              </button>
            </Tooltip>
            <Tooltip
              mouseLeaveDelay={0}
              arrow={false}
              title={t('userhome.profile.lock')}
              placement="bottom"
            >
              <button className="userhome-profile-button-lock" onClick={() => invoke(SeelenCommand.Lock)}>
                <Icon iconName="BiLock" />
              </button>
            </Tooltip>
          </div>
          <Tooltip
            mouseLeaveDelay={0}
            arrow={false}
            title={t('placeholder.open_user_folder')}
            placement="bottom">
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
            title={t('userhome.profile.open-one-drive')}
            placement="bottom">
            <button
              className="userhome-profile-onedrive"
              onClick={() => invoke(SeelenCommand.OpenFile, { path: user.oneDrivePath })}
            >
              <Icon iconName="TbBrandOnedrive" />
              <div className="userhome-profile-onedrive-text">{t('userhome.profile.one-drive')}</div>
            </button>
          </Tooltip>
          <Tooltip
            mouseLeaveDelay={0}
            arrow={false}
            title={t('userhome.profile.passwords')}
            placement="bottom">
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
      { recentFiles.length != 0 &&
        <>
          <div className="userhome-title">{t('userhome.recent_files.title')}</div>
          <ul className="userhome-history">
            { recentFiles.slice(0, recentCount).map((item, index) => (
              <Tooltip
                key={index}
                mouseLeaveDelay={0}
                arrow={false}
                title={item.path}
                placement="top">
                <li className="userhome-history-item">
                  <img className="userhome-history-item-icon" src={convertFileSrc(item.iconLocation)} />
                  <div className="userhome-history-item-title">{item.name}</div>
                  <div className="userhome-history-item-date" >{moment(WindowsDateFileTimeToDate(item.lastAccessTime)).fromNow()}</div>
                </li>
              </Tooltip>
            ))}
          </ul>
          { recentFiles.length > 5 &&
            <button onClick={() => setRecentCount(recentFiles.length > recentCount ? recentCount * 2 : 5)}>{recentFiles.length > recentCount ? t('userhome.history.more-items') : t('userhome.history.reduce-items')}</button>
          }
        </>
      }
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
                placement="top">
                <li className="userhome-history-item" onClick={() => invoke(SeelenCommand.RequestFocus, { hwnd: item.hwnd })}>
                  <img className="userhome-history-item-icon" src={convertFileSrc(item.icon_path)} />
                  <div className="userhome-history-item-title">{item.name} - {item.title}</div>
                  <div className="userhome-history-item-date" >{item.date.fromNow()}</div>
                </li>
              </Tooltip>
            ))}
          </ul>
          { history.length > 5 &&
            <button onClick={() => setHistoryCount(history.length > historyCount ? historyCount * 2 : 5)}>{history.length > historyCount ? t('userhome.history.more-items') : t('userhome.history.reduce-items')}</button>
          }
        </>
      }
    </BackgroundByLayersV2>
  );
}

export function WithUserHome({ children }: PropsWithChildren) {
  const [openPreview, setOpenPreview] = useState(false);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

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
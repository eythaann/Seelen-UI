import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { User } from '@seelen-ui/lib/types';
import { path } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Icon } from 'src/apps/shared/components/Icon';

interface Props {
  user: User;
}

export function UserProfile({ user }: Props) {
  const { t } = useTranslation();

  return (
    <div className="userhome-profile-container">
      <div className="userhome-profile-picture-container">
        <img
          className="userhome-profile-picture"
          src={convertFileSrc(user.profilePicturePath ?? LAZY_CONSTANTS.MISSING_ICON_PATH)}
        />
        <Tooltip
          mouseLeaveDelay={0}
          arrow={false}
          title={t('settings.log_out')}
          placement="bottom"
        >
          <button
            className="userhome-profile-lock-button"
            onClick={() => invoke(SeelenCommand.LogOut)}
          >
            <Icon iconName="BiLogOut" />
          </button>
        </Tooltip>
        <Tooltip
          mouseLeaveDelay={0}
          arrow={false}
          title={t('userhome.profile.accounts')}
          placement="bottom"
        >
          <button
            className="userhome-profile-settings-button"
            onClick={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:accounts' })}
          >
            <Icon iconName="RiSettings3Fill" />
          </button>
        </Tooltip>
      </div>

      <div className="userhome-profile-information">
        <div className="userhome-profile-name">
          <span>{user.name}</span>
          <Tooltip
            mouseLeaveDelay={0}
            arrow={false}
            title={t('userhome.profile.open_user_folder')}
            placement="right"
          >
            <button
              className="userhome-profile-action-button"
              onClick={async () => {
                invoke(SeelenCommand.OpenFile, { path: await path.homeDir() });
              }}
            >
              <Icon iconName="PiFolderUser" />
            </button>
          </Tooltip>
        </div>

        <div className="userhome-profile-email">{user.email}</div>

        { user.oneDrivePath &&
          <Tooltip
            mouseLeaveDelay={0}
            arrow={false}
            title={t('userhome.profile.open_onedrive')}
            placement="right"
          >
            <button
              className="userhome-profile-action-button"
              onClick={() => invoke(SeelenCommand.OpenFile, { path: user.oneDrivePath })}
            >
              <Icon iconName="ImOnedrive" />
              <span>OneDrive</span>
            </button>
          </Tooltip>
        }
      </div>
    </div>
  );
}

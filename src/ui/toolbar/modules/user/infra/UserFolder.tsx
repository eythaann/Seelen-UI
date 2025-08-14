import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { FolderType } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { cx } from '@shared/styles';
import { path } from '@tauri-apps/api';
import { t } from 'i18next';
import { useState } from 'react';

import { EmptyList } from './EmptyList';
import { FilePreview } from './FilePreview';
import { UserHomeFolder } from './UserHome';

interface UserFolderProps {
  folderProps: UserHomeFolder;
  categoryOpen: FolderType;
  setCategoryOpen: (category: FolderType) => void;
}

const pathByCategory: Record<FolderType, string> = {
  Recent: (await path.dataDir()) + '\\Microsoft\\Windows\\Recent',
  Desktop: await path.desktopDir(),
  Documents: await path.documentDir(),
  Downloads: await path.downloadDir(),
  Music: await path.audioDir(),
  Pictures: await path.pictureDir(),
  Videos: await path.videoDir(),
  Unknown: await path.homeDir(),
};

export function UserFolder({ folderProps, setCategoryOpen, categoryOpen }: UserFolderProps) {
  const [folderShowCount, setFolderShowCount] = useState(5);
  const { content, category, icon } = folderProps;

  const OpenOnExplorer = async () => {
    invoke(SeelenCommand.OpenFile, { path: pathByCategory[category] });
  };

  const onClickChevron = (e: MouseEvent) => {
    e.stopPropagation();
    setCategoryOpen(categoryOpen == category ? 'Unknown' : category);
  };

  const files =
    category == 'Recent'
      ? content.filter((item) => !item.path.toLocaleLowerCase().endsWith('.lnk'))
      : content;

  return (
    <div className="userhome-directory">
      <div className="userhome-directory-header" onClick={OpenOnExplorer}>
        <Icon iconName={icon} className="userhome-directory-icon"></Icon>
        <span>{t(`userhome.folders.${category.toLowerCase()}`)}</span>
        <button className="userhome-directory-header-collapse-button" onClick={onClickChevron}>
          <Icon
            iconName="IoIosArrowDown"
            className={cx('chevron', {
              'chevron-active': category == categoryOpen,
            })}
          />
        </button>
      </div>
      <ul
        className={cx('userhome-directory-content', {
          'userhome-directory-content-open': category == categoryOpen,
        })}
      >
        {files.length == 0 && <EmptyList />}
        {files.slice(0, folderShowCount).map((item, index) => (
          <FilePreview file={item} key={index} />
        ))}
        {files.length > 5 && (
          <button
            className="userhome-list-extender"
            onClick={(e) => {
              setFolderShowCount(files.length > folderShowCount ? folderShowCount * 2 : 5);
              e.stopPropagation();
            }}
          >
            {files.length > folderShowCount
              ? t('userhome.folders.more_items')
              : t('userhome.folders.reduce_items')}
          </button>
        )}
      </ul>
    </div>
  );
}

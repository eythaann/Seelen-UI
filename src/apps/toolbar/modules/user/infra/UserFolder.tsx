import { FolderType } from '@seelen-ui/lib/types';
import { t } from 'i18next';
import { useState } from 'react';

import { cx } from 'src/apps/shared/styles';

import { Icon } from '../../../../shared/components/Icon';
import { FilePreview } from './FilePreview';
import { UserHomeFolder } from './UserHome';

interface UserFolderProps {
  folderProps: UserHomeFolder;
  categoryOpen: FolderType;
  setCategoryOpen: (category: FolderType) => void;
}

export function UserFolder({ folderProps, setCategoryOpen, categoryOpen }: UserFolderProps) {
  const [folderShowCount, setFolderShowCount] = useState(5);
  const { content, category, icon } = folderProps;

  return (
    content && content.length != 0 &&
    <li className="userhome-folder-category" onClick={() => setCategoryOpen(categoryOpen == category ? 'Unknown' : category)}>
      <div className="userhome-folder-title">
        <Icon iconName={icon} className="userhome-folder-icon"></Icon>
        <span>{t(`userhome.${category.toLowerCase()}.title`)}</span>
        <Icon iconName="IoIosArrowDown" className={cx('userhome-folder-expander', { 'userhome-folder-expander-open': category == categoryOpen })} ></Icon>
      </div>
      <ul className={cx('userhome-folder-content', { 'userhome-folder-content-open': category == categoryOpen })}>
        { content.slice(0, folderShowCount).map((item, index) => (
          <FilePreview file={item} key={index} />
        ))}
        { content.length > 5 &&
          <button className="userhome-folder-viewcount-extender" onClick={(e) => {
            setFolderShowCount(content.length > folderShowCount ? folderShowCount * 2 : 5);
            e.stopPropagation();
          }}>{content.length > folderShowCount ? t(`userhome.${category.toLowerCase()}.more-items`) : t(`userhome.${category.toLowerCase()}.reduce-items`)}</button>
        }
      </ul>
    </li>
  );
}
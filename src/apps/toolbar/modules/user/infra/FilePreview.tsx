import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { File } from '@seelen-ui/lib/types';
import { Tooltip } from 'antd';
import moment from 'moment';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { useIcon } from 'src/apps/shared/hooks';

interface FilePreviewProps {
  file: File;
}

const EPOCH_DIFF_MILLISECONDS = 11644473600000n;

function WindowsDateFileTimeToDate(fileTime: bigint) {
  return new Date(Number(BigInt(fileTime) / 10000n - EPOCH_DIFF_MILLISECONDS));
}

export function FilePreview({ file }: FilePreviewProps) {
  const icon = useIcon({ path: file.path });

  return (
    <Tooltip
      mouseLeaveDelay={0}
      arrow={false}
      title={file.path}
      placement="right"
    >
      <li className="userhome-folder-content-item" onClick={() => invoke(SeelenCommand.OpenFile, { path: file.path })}>
        <img className="userhome-folder-content-item-icon" src={icon || LAZY_CONSTANTS.MISSING_ICON_PATH} />
        <div className="userhome-folder-content-item-title">{file.path.substring(file.path.lastIndexOf('\\') + 1)}</div>
        <div className="userhome-folder-content-item-date" >{moment(WindowsDateFileTimeToDate(file.lastAccessTime)).fromNow()}</div>
      </li>
    </Tooltip>
  );
}
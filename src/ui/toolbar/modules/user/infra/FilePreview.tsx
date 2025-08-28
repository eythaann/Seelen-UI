import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { File } from '@seelen-ui/lib/types';
import { WindowsDateFileTimeToDate } from '@shared';
import { FileIcon } from '@shared/components/Icon';
import { Tooltip } from 'antd';
import moment from 'moment';

interface FilePreviewProps {
  file: File;
}

export function FilePreview({ file }: FilePreviewProps) {
  return (
    <Tooltip mouseLeaveDelay={0} arrow={false} title={file.path} placement="right">
      <li
        className="userhome-file"
        onClick={() => invoke(SeelenCommand.SelectFileOnExplorer, { path: file.path })}
      >
        <FileIcon className="userhome-file-icon" path={file.path} />
        <div className="userhome-file-label">
          {file.path.substring(file.path.lastIndexOf('\\') + 1)}
        </div>
        <div className="userhome-file-date">
          {moment(WindowsDateFileTimeToDate(file.lastAccessTime)).fromNow()}
        </div>
      </li>
    </Tooltip>
  );
}

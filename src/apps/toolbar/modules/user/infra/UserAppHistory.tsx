// TODO(lurius): use this file on another module
import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { ApplicationHistoryEntry } from '@seelen-ui/lib/types';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import moment from 'moment';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';
import { Icon } from 'src/apps/shared/components/Icon';
import { useIcon } from 'src/apps/shared/hooks';
import { cx } from 'src/apps/shared/styles';

import { EmptyList } from './EmptyList';

const filter = (history: ApplicationHistoryEntry[]) => {
  return (
    history
      // Do not want to show the Seelen items
      .filter((item) => !item.isSeelen)
      // Filter those which are there bacause the Seelen are filtered out
      .filter(
        (item, index, array) =>
          index == 0 || array[index - 1]?.application.hwnd != item.application.hwnd,
      )
      // Only show the last 20 of the history max!
      .slice(0, 20)
  );
};

interface HistoryItemProps {
  item: ApplicationHistoryEntry;
}

function HistoryItem({ item }: HistoryItemProps) {
  const iconSrc =
    useIcon({ path: item.application.exe, umid: item.application.umid }) ||
    convertFileSrc(LAZY_CONSTANTS.MISSING_ICON_PATH);

  function onClick() {
    // idea: make the used app start if not open, like seelen dock
    invoke(SeelenCommand.RequestFocus, { hwnd: item.application.hwnd });
  }

  return (
    <Tooltip
      mouseLeaveDelay={0}
      arrow={false}
      title={item.application.name + ' - ' + item.application.title}
      placement="right"
    >
      <li className="userhome-file" onClick={onClick}>
        <img className="userhome-file-icon" src={iconSrc} />
        <div className="userhome-file-label">
          {item.application.name} - {item.application.title}
        </div>
        <div className="userhome-file-date">
          {moment(new Date(Number(item.focusDate))).fromNow()}
        </div>
      </li>
    </Tooltip>
  );
}

export function UserAppHistory() {
  const [historyIsLocal, setHistoryIsLocal] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [amountToShow, setAmountToShow] = useState(5);

  const storeHistory = useSelector(Selectors.history);
  const storeOnMonitorHistory = useSelector(Selectors.historyOnMonitor);

  const { t } = useTranslation();

  const history = historyIsLocal ? filter(storeOnMonitorHistory) : filter(storeHistory);

  return (
    <div>
      <span
        className="userhome-label userhome-history-label"
        onClick={() => setHistoryOpen(!historyOpen)}
      >
        <span>{t('userhome.history.title')}</span>
        <Tooltip
          mouseLeaveDelay={0}
          arrow={false}
          title={historyIsLocal ? t('userhome.history.local') : t('userhome.history.global')}
          placement="right"
        >
          <Icon
            iconName={historyIsLocal ? 'PiAppWindowLight' : 'AiOutlineGlobal'}
            className="userhome-history-location"
            onClick={(e) => {
              setHistoryIsLocal(!historyIsLocal);
              e.preventDefault();
              e.stopPropagation();
            }}
          />
        </Tooltip>
        <Icon
          iconName="IoIosArrowDown"
          className={cx('chevron', {
            'chevron-active': historyOpen,
          })}
        />
      </span>
      <ul className={cx('userhome-history-list', { 'userhome-history-list-open': historyOpen })}>
        {!history.length && <EmptyList />}
        {!historyIsLocal &&
          filter(storeHistory)
            .slice(0, amountToShow)
            .map((item, index) => <HistoryItem key={index} item={item} />)}
        {historyIsLocal &&
          filter(storeOnMonitorHistory)
            .slice(0, amountToShow)
            .map((item, index) => <HistoryItem key={index} item={item} />)}
        {history.length > 5 && (
          <button
            className="userhome-list-extender"
            onClick={() => setAmountToShow(amountToShow < history.length ? amountToShow * 2 : 5)}
          >
            {amountToShow < history.length
              ? t('userhome.history.more_items')
              : t('userhome.history.reduce_items')}
          </button>
        )}
      </ul>
    </div>
  );
}

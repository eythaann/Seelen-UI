import { IconPackManager, SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Checkbox, Spin, Tooltip } from 'antd';
import { motion } from 'framer-motion';
import { KeyboardEventHandler, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { Selectors } from '../../shared/store/app';
import { SaveHistory } from '../app';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { CommandInput } from './CommandInput';
import { Item } from './Item';
import { RunnerSelector } from './RunnerSelector';

export function Launcher() {
  const [loading, setLoading] = useState(true);
  const [showHelp, setShowHelp] = useState(true);
  const [showHistory, setShowHistory] = useState(false);
  const [_command, _setCommand] = useState('');
  const [usingRunnerIdx, setUsingRunnerIdx] = useState(0);

  const history = useSelector(Selectors.history);
  const runners = useSelector(Selectors.settings.runners);
  const apps = useSelector(Selectors.apps);

  const selectorRef = useRef<HTMLInputElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const helpRef = useRef<HTMLInputElement>(null);

  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (focused) {
      inputRef.current?.focus();
    } else {
      _setCommand('');
      getCurrentWindow().hide();
    }
  });

  useEffect(() => {
    async function loadIcons() {
      for (const app of apps) {
        await IconPackManager.requestIconExtraction(app);
      }
    }
    // we load all icons one by one first to avoid load all icons at the same time and block the UI
    loadIcons().finally(() => setLoading(false));
  }, []);

  const command = _command.trim().toLowerCase();
  const selectedRunner = runners[usingRunnerIdx];
  const selectedHistory = selectedRunner ? history[selectedRunner.id] || [] : [];
  const matchingHistory = selectedHistory
    .filter((value) => value.toLowerCase().includes(command))
    .map((value) => ({ value }));

  const onInputKeyDown: KeyboardEventHandler<HTMLInputElement> = (e) => {
    if (!showHistory || matchingHistory.length === 0) {
      if (e.key === 'Enter') {
        invoke(SeelenCommand.OpenFile, { path: command });
        getCurrentWindow().hide();
        if (selectedRunner) {
          SaveHistory({
            ...history,
            [selectedRunner.id]: [...new Set([command, ...(history[selectedRunner.id] || [])])],
          });
        }
        return;
      }
    }
  };

  const onDocumentKeyDown: KeyboardEventHandler<HTMLInputElement> = (e) => {
    if (e.ctrlKey && e.key === 'Tab') {
      setUsingRunnerIdx(
        (current) => (e.shiftKey ? current + runners.length - 1 : current + 1) % runners.length,
      );
      return;
    }

    if (e.ctrlKey && e.key === 'h') {
      setShowHelp(!showHelp);
      return;
    }

    if (e.ctrlKey && e.key === 'f') {
      inputRef.current?.focus();
      return;
    }
  };

  return (
    <motion.div className="launcher" onKeyDown={onDocumentKeyDown}>
      <div className="launcher-header">
        <RunnerSelector
          ref={selectorRef}
          selectedRunner={usingRunnerIdx}
          runners={runners}
          setSelectedRunner={setUsingRunnerIdx}
          helpRef={helpRef}
          showHelp={showHelp}
        />
        <CommandInput
          command={_command}
          setCommand={_setCommand}
          showHistory={showHistory}
          setShowHistory={setShowHistory}
          matchingHistory={matchingHistory}
          onInputKeyDown={onInputKeyDown}
          inputRef={inputRef}
          showHelp={showHelp}
        />
      </div>
      <Tooltip open={showHelp} title="Tab / Shift + Tab" placement="left">
        <div className="launcher-body">
          {loading ? (
            <div className="launcher-loading">
              <Spin size="large" />
            </div>
          ) : (
            apps.map((item) => (
              <Item
                key={item.path}
                item={item}
                hidden={!item.path.toLowerCase().includes(command)}
              />
            ))
          )}
        </div>
      </Tooltip>
      <div className="launcher-footer">
        <Checkbox
          ref={helpRef as any}
          checked={showHelp}
          onChange={(e) => setShowHelp(e.target.checked)}
          onKeyDown={(e) => {
            if (e.key === 'Tab') {
              selectorRef.current?.focus();
              e.preventDefault();
            }
          }}
        >
          <Tooltip open={showHelp} title="Ctrl + H" placement="right">
            {t('footer.shortcuts')}
          </Tooltip>
        </Checkbox>
      </div>
    </motion.div>
  );
}

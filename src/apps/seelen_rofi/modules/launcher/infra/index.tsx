import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { AutoComplete, Checkbox, Select, Tooltip } from 'antd';
import { motion } from 'framer-motion';
import { KeyboardEventHandler, useRef, useState } from 'react';
import { useSelector } from 'react-redux';
import { useWindowFocusChange } from 'seelen-core';

import { Selectors } from '../../shared/store/app';
import { SaveHistory } from '../app';

import { Item } from './Item';

export function Launcher() {
  const [showHelp, setShowHelp] = useState(true);
  const [showHistory, setShowHistory] = useState(false);
  const [_command, _setCommand] = useState('');
  const [selectedRunner, setSelectedRunner] = useState(0);

  const history = useSelector(Selectors.history);
  const runners = useSelector(Selectors.settings.runners);
  const apps = useSelector(Selectors.apps);

  const selectorRef = useRef<HTMLInputElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const helpRef = useRef<HTMLInputElement>(null);

  useWindowFocusChange((focused) => {
    if (focused) {
      inputRef.current?.focus();
    } else {
      _setCommand('');
      getCurrentWindow().hide();
    }
  });

  const onInputKeyDown: KeyboardEventHandler<HTMLInputElement> = (e) => {
    if (!showHistory || matchingHistory.length === 0) {
      if (e.key === 'Enter') {
        invoke('open_file', { path: command });
        getCurrentWindow().hide();
        SaveHistory({
          ...history,
          [selectedRunner]: [...new Set([command, ...(history[selectedRunner] || [])])],
        });
        return;
      }
    }
  };

  const onDocumentKeyDown: KeyboardEventHandler<HTMLInputElement> = (e) => {
    if (e.ctrlKey && e.key === 'Tab') {
      setSelectedRunner(
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

  const command = _command.trim().toLowerCase();
  const selectedHistory = history[selectedRunner] || [];
  const matchingHistory = selectedHistory
    .filter((value) => value.toLowerCase().includes(command))
    .map((value) => ({ value }));

  return (
    <motion.div className="launcher" onKeyDown={onDocumentKeyDown}>
      <div className="launcher-header">
        <Tooltip open={showHelp} title="Ctrl + Tab" placement="left">
          <Select
            ref={selectorRef as any}
            className="launcher-header-runner-selector"
            value={selectedRunner}
            onChange={setSelectedRunner}
            options={runners.map((runner, idx) => ({
              key: runner.id,
              label: runner.label,
              value: idx,
            }))}
            onKeyDown={(e) => {
              if (e.shiftKey && e.key === 'Tab') {
                helpRef.current?.focus();
                e.preventDefault();
              }
            }}
          />
        </Tooltip>
        <Tooltip open={showHelp} title="Ctrl + F" placement="top">
          <Tooltip open={showHelp} title="Enter" placement="right">
            <AutoComplete
              ref={inputRef as any}
              className="launcher-header-command-input"
              placeholder="App, Command or Path..."
              options={matchingHistory}
              filterOption
              value={_command}
              onChange={_setCommand}
              open={showHistory}
              onDropdownVisibleChange={setShowHistory}
              onInputKeyDown={onInputKeyDown}
              autoFocus
              allowClear
            />
          </Tooltip>
        </Tooltip>
      </div>
      <Tooltip open={showHelp} title="Tab / Shift + Tab" placement="left">
        <div className="launcher-body">
          {apps.map((item) => (
            <Item
              key={item.executionPath}
              item={item}
              hidden={item.label.toLowerCase().includes(command)}
            />
          ))}
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
            Show Shortcuts
          </Tooltip>
        </Checkbox>
      </div>
    </motion.div>
  );
}

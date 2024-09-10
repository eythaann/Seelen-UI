import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { AutoComplete, Checkbox, Dropdown, Menu, Select, Tooltip } from 'antd';
import { motion } from 'framer-motion';
import { KeyboardEventHandler, useRef, useState } from 'react';
import { useSelector } from 'react-redux';
import { useWindowFocusChange } from 'seelen-core';

import { Selectors } from '../shared/store/app';
import { SaveHistory } from './app';
import { OverflowTooltip } from 'src/apps/shared/components/OverflowTooltip';

interface Item {
  label: string;
  icon: string;
  path: string;
  executionPath: string;
}

export function Item(props: { item: Item }) {
  const {
    item: { label, icon, executionPath, path },
  } = props;

  function onClick() {
    invoke('open_file', { path: executionPath });
    getCurrentWindow().hide();
  }

  let shortPath = executionPath.slice(executionPath.indexOf('\\Programs\\') + 10);
  return (
    <Dropdown
      trigger={['contextMenu']}
      dropdownRender={() => (
        <Menu
          items={[
            {
              label: 'Open File Location',
              key: 'open',
              onClick() {
                invoke('select_file_on_explorer', { path });
              },
            },
          ]}
        />
      )}
    >
      <button className="launcher-item" onClick={onClick}>
        <img className="launcher-item-icon" src={convertFileSrc(icon)} />
        <OverflowTooltip className="launcher-item-label" text={label} />
        <OverflowTooltip className="launcher-item-path" text={shortPath} />
      </button>
    </Dropdown>
  );
}

export function Launcher() {
  const [showHelp, setShowHelp] = useState(true);
  const [showHistory, setShowHistory] = useState(false);
  const [command, setCommand] = useState('');
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
      setCommand('');
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

  const selectedHistory = history[selectedRunner] || [];
  const matchingHistory = selectedHistory
    .filter((value) => value.toLowerCase().includes(command.toLowerCase()))
    .map((value) => ({ value }));

  const items = apps.filter((item) => item.label.toLowerCase().includes(command.toLowerCase()));

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
              value={command}
              onChange={setCommand}
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
          {items.map((item) => (
            <Item key={item.executionPath} item={item} />
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

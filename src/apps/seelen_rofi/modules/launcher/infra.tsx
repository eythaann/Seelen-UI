import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { AutoComplete, Checkbox, Dropdown, Menu, Select, Tooltip } from 'antd';
import { motion } from 'framer-motion';
import { KeyboardEventHandler, useRef, useState } from 'react';
import { useSelector } from 'react-redux';
import { useWindowFocusChange } from 'seelen-core';

import { Selectors } from '../shared/store/app';
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
    invoke('open_file', { executionPath });
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

enum Runner {
  Run = 'run',
  Command = 'command',
}

const history: Record<Runner, string[]> = {
  [Runner.Run]: ['%temp%', 'shell:AppsFolder', '%appdata%'],
  [Runner.Command]: ['DEL /A /Q "%localappdata%\\IconCache.db"'],
};

export function Launcher() {
  const [showHelp, setShowHelp] = useState(true);
  const [showHistory, setShowHistory] = useState(false);
  const [command, setCommand] = useState('');
  const [runner, setRunner] = useState(Runner.Run);

  const apps = useSelector(Selectors.apps);

  const input = useRef<HTMLInputElement>(null);

  useWindowFocusChange((focused) => {
    if (focused) {
      input.current?.focus();
    } else {
      setCommand('');
      getCurrentWindow().hide();
    }
  });

  const nextRunner = () => {
    let runners = Object.values(Runner);
    setRunner(runners[(runners.indexOf(runner) + 1) % runners.length]!);
  };

  const matchingHistory = history[runner]
    .map((path) => ({
      label: path,
      value: path,
    }))
    .filter((option) => option.label.toLowerCase().includes(command.toLowerCase()));

  const items = apps.filter((item) => item.label.toLowerCase().includes(command.toLowerCase()));

  const onKeyDown: KeyboardEventHandler<HTMLInputElement> = (e) => {
    if (e.ctrlKey && e.key === 'Tab') {
      nextRunner();
      return;
    }

    if (e.ctrlKey && e.key === 'h') {
      setShowHelp(!showHelp);
      return;
    }

    if (e.ctrlKey && e.key === 's') {
      input.current?.focus();
      return;
    }

    if (!showHistory || matchingHistory.length === 0) {
      if (e.key === 'Enter') {
        invoke('open_file', { path: command });
        getCurrentWindow().hide();
        return;
      }
    }
  };

  return (
    <motion.div className="launcher" onKeyDown={onKeyDown}>
      <div className="launcher-header">
        <Tooltip open={showHelp} title="Ctrl + Tab" placement="left">
          <Select
            className="launcher-header-runner-selector"
            value={runner}
            onChange={setRunner}
            options={Object.values(Runner).map((runner) => ({
              label: runner,
              value: runner,
            }))}
          />
        </Tooltip>
        <Tooltip open={showHelp} title="Ctrl + S" placement="top">
          <Tooltip open={showHelp} title="Enter" placement="right">
            <AutoComplete
              ref={input as any}
              className="launcher-header-command-input"
              placeholder="App, Command or Path..."
              options={matchingHistory}
              filterOption
              value={command}
              onChange={setCommand}
              open={showHistory}
              onDropdownVisibleChange={setShowHistory}
              autoFocus
              allowClear
            />
          </Tooltip>
        </Tooltip>
      </div>
      <div className="launcher-body">
        {items.map((item) => (
          <Item key={item.executionPath} item={item} />
        ))}
      </div>
      <div className="launcher-footer">
        <Checkbox checked={showHelp} onChange={(e) => setShowHelp(e.target.checked)}>
          <Tooltip open={showHelp} title="Ctrl + H" placement="right">
            Show Shortcuts
          </Tooltip>
        </Checkbox>
      </div>
    </motion.div>
  );
}

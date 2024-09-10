import { AutoComplete, Tooltip } from 'antd';
import { KeyboardEventHandler, RefObject } from 'react';

interface CommandInputProps {
  command: string;
  setCommand: (value: string) => void;
  showHistory: boolean;
  setShowHistory: (value: boolean) => void;
  matchingHistory: Array<{ value: string }>;
  onInputKeyDown: KeyboardEventHandler<HTMLInputElement>;
  inputRef: RefObject<HTMLInputElement>;
  showHelp: boolean;
}

export const CommandInput = ({
  command,
  setCommand,
  showHistory,
  setShowHistory,
  matchingHistory,
  onInputKeyDown,
  inputRef,
  showHelp,
}: CommandInputProps) => (
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
);
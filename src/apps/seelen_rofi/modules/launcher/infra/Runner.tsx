import { Select, Tooltip } from 'antd';
import { RefObject } from 'react';

interface RunnerSelectorProps {
  selectedRunner: number;
  runners: Array<{ id: string; label: string }>;
  setSelectedRunner: (value: number) => void;
  helpRef: RefObject<HTMLInputElement>;
  showHelp: boolean;
}

export const RunnerSelector = ({
  selectedRunner,
  runners,
  setSelectedRunner,
  helpRef,
  showHelp,
}: RunnerSelectorProps) => {
  return (
    <Tooltip open={showHelp} title="Ctrl + Tab" placement="left">
      <Select
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
  );
};

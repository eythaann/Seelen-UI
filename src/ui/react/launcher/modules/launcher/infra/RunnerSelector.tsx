import { Select, Tooltip } from "antd";
import { forwardRef, type RefObject } from "react";
import { useTranslation } from "react-i18next";

interface RunnerSelectorProps {
  selectedRunner: number;
  runners: Array<{ id: string; label: string }>;
  setSelectedRunner: (value: number) => void;
  helpRef: RefObject<HTMLInputElement>;
  showHelp: boolean;
}

export const RunnerSelector = forwardRef((props: RunnerSelectorProps, ref) => {
  const { selectedRunner, runners, setSelectedRunner, helpRef, showHelp } = props;

  const { t } = useTranslation();

  return (
    <Tooltip open={showHelp} title="Ctrl + Tab" placement="left">
      <Select
        ref={ref as any}
        className="launcher-header-runner-selector"
        value={selectedRunner}
        onChange={setSelectedRunner}
        options={runners.map((runner, idx) => ({
          key: runner.id,
          label: runner.label.startsWith("t:") ? t(runner.label.replace("t:", "")) : runner.label,
          value: idx,
        }))}
        onKeyDown={(e) => {
          if (e.shiftKey && e.key === "Tab") {
            helpRef.current?.focus();
            e.preventDefault();
          }
        }}
      />
    </Tooltip>
  );
});

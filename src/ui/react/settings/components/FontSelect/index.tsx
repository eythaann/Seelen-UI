import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { Select } from "antd";

interface Props {
  value?: string;
  defaultValue?: string;
  onChange?: (value: string) => void;
  style?: React.CSSProperties;
}

const fonts = await invoke(SeelenCommand.GetFonts);

export function FontSelect({ value, defaultValue, onChange, style }: Props) {
  const options = fonts.map((f) => ({
    value: f.family,
    label: <span style={{ fontFamily: f.family }}>{f.family}</span>,
  }));

  return (
    <Select
      showSearch={{ optionFilterProp: "value" }}
      allowClear
      value={value}
      defaultValue={defaultValue}
      onChange={onChange}
      options={options}
      style={{ minWidth: 200, ...style }}
    />
  );
}

import { InputNumber, Select } from "antd";
import { useTranslation } from "react-i18next";
import { useDispatch } from "react-redux";

import { useAppSelector } from "../../../shared/utils/infra.ts";

import { SeelenWmSelectors } from "../../../shared/store/app/selectors.ts";
import { WManagerSettingsActions } from "../app.ts";

import { SettingsGroup, SettingsOption } from "../../../../components/SettingsBox/index.tsx";
import { WmDragBehavior } from "@seelen-ui/lib/types";

export const OthersConfigs = () => {
  const resizeDelta = useAppSelector(SeelenWmSelectors.resizeDelta);
  const dragBehavior = useAppSelector(SeelenWmSelectors.dragBehavior);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const onChangeResizeDelta = (value: number | null) => {
    dispatch(WManagerSettingsActions.setResizeDelta(value || 0));
  };

  const onChangeDragBehavior = (value: WmDragBehavior) => {
    dispatch(WManagerSettingsActions.setDragBehavior(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("wm.drag_behavior")}</b>
          <Select
            style={{ width: "200px" }}
            value={dragBehavior}
            options={[
              {
                label: t("wm.drag_behavior_options.sort"),
                value: WmDragBehavior.Sort,
              },
              {
                label: t("wm.drag_behavior_options.swap"),
                value: WmDragBehavior.Swap,
              },
            ]}
            onSelect={onChangeDragBehavior}
          />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t("wm.resize_delta")}</b>
          <InputNumber value={resizeDelta} onChange={onChangeResizeDelta} min={1} max={40} />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
};

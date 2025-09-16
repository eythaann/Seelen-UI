import { Modal } from "antd";

import { SaveStore, store } from "../shared/store/infra";
import { startup } from "../shared/tauri/infra";

import { RootActions } from "../shared/store/app/reducer";

import i18n from "../../i18n";
import cs from "./index.module.css";

export const StartUser = () => {
  startup.enable();
  store.dispatch(RootActions.setAutostart(true));

  const modal = Modal.confirm({
    title: i18n.t("start.title"),
    className: cs.welcome,
    content: (
      <div>
        <p>
          {i18n.t("start.message")}
        </p>
        <b>{i18n.t("start.message_accent")}</b>
      </div>
    ),
    okText: "Continue",
    onOk: () => {
      SaveStore();
      modal.destroy();
    },
    icon: <div className={cs.icon}>🎉</div>,
    cancelButtonProps: { style: { display: "none" } },
    centered: true,
  });
};

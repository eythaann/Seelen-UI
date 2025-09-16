import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { AppNotification } from "@seelen-ui/lib/types";
import { AnimatePresence, motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { Selectors } from "../../shared/store/app";

import { Notification } from "./Notification";

export function Notifications() {
  const notifications = useSelector(Selectors.notifications);
  console.log({ notifications });

  const { t } = useTranslation();

  return (
    <BackgroundByLayersV2
      className="notifications"
      onContextMenu={(e) => e.stopPropagation()}
    >
      <div className="notifications-header">
        <span>{t("notifications.title")}</span>
        <button
          className="notifications-clear-button"
          onClick={() => {
            invoke(SeelenCommand.NotificationsCloseAll).catch(console.error);
          }}
        >
          {t("notifications.clear")}
        </button>
      </div>

      <div className="notifications-body">
        <AnimatePresence>
          {notifications.map((notification: AppNotification) => (
            <Notification key={notification.id} notification={notification} />
          ))}
        </AnimatePresence>

        {!notifications.length && (
          <motion.div
            className="notifications-empty"
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 200 }}
            transition={{ duration: 0.2, delay: 0.4 }}
          >
            <p>{t("notifications.empty")}</p>
          </motion.div>
        )}
      </div>
      <div className="notifications-footer">
        <button
          className="notifications-settings-button"
          onClick={() => {
            invoke(SeelenCommand.OpenFile, {
              path: "ms-settings:notifications",
            }).catch(
              console.error,
            );
          }}
        >
          {t("notifications.settings")}
        </button>
      </div>
    </BackgroundByLayersV2>
  );
}

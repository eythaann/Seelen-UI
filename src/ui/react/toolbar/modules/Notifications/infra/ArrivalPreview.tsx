import type { AppNotification } from "@seelen-ui/lib/types";
import { WindowsDateFileTimeToDate } from "@shared";
import { useInterval } from "@shared/hooks";
import { AnimatePresence } from "framer-motion";
import { useEffect, useState } from "react";
import { useSelector } from "react-redux";

import { Selectors } from "../../shared/store/app.ts";

import { Notification } from "./Notification.tsx";

const NOTIFICATION_PREVIEW_TIME = 5 * 1000;

export function ArrivalPreview() {
  const allNotifications = useSelector(Selectors.notifications);
  const [arrivals, setArrivals] = useState<Record<string, AppNotification>>({});

  const updateArrivals = () => {
    for (const notification of allNotifications) {
      const arrivalDate = WindowsDateFileTimeToDate(BigInt(notification.date));
      if (Date.now() - arrivalDate.getTime() < NOTIFICATION_PREVIEW_TIME) {
        setArrivals((prev) => ({
          ...prev,
          [`${notification.id}`]: notification,
        }));
      }
    }
  };

  const cleanArrivals = () => {
    setArrivals((current) => {
      const newState = { ...current };
      for (const key in newState) {
        const arrivalDate = WindowsDateFileTimeToDate(
          BigInt(newState[key]!.date),
        );
        if (Date.now() - arrivalDate.getTime() > NOTIFICATION_PREVIEW_TIME) {
          delete newState[key];
        }
      }
      return newState;
    });
  };

  useEffect(() => {
    updateArrivals();
    setTimeout(() => cleanArrivals(), 5000);
  }, [allNotifications]);

  useInterval(() => {
    cleanArrivals();
  }, NOTIFICATION_PREVIEW_TIME);

  return (
    <div
      className="notification-arrival"
      onContextMenu={(e) => e.stopPropagation()}
    >
      <AnimatePresence>
        {Object.values(arrivals).map((notification) => (
          <Notification
            key={notification.id}
            notification={notification}
            onClose={() => {
              setArrivals((current) => {
                const newState = { ...current };
                delete newState[notification.id];
                return newState;
              });
            }}
          />
        ))}
      </AnimatePresence>
    </div>
  );
}

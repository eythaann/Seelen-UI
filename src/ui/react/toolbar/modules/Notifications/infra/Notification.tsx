import { useSignal } from "@preact/signals";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import type {
  AppNotification,
  ToastActionActivationType,
  ToastActionsChild,
  ToastBindingChild,
  ToastImage,
} from "@seelen-ui/lib/types";
import { WindowsDateFileTimeToDate } from "libs/ui/react/utils";
import { FileIcon, Icon } from "libs/ui/react/components/Icon";
import { cx } from "libs/ui/react/utils/styling";
import { Select, Tooltip } from "antd";
import { motion } from "framer-motion";
import moment from "moment";

interface Props {
  notification: AppNotification;
  onClose?: () => void;
}

export function Notification({ notification, onClose }: Props) {
  const $data = useSignal<Record<string, string>>({});

  const { logoImage, heroImage, body, actions } = splitToastContent(
    notification,
  );

  function onInputChange(key: string, value: string) {
    $data.value = { ...$data.value, [key]: value };
  }

  function onAction(args: string, method: ToastActionActivationType) {
    switch (method) {
      case "Protocol":
        invoke(SeelenCommand.OpenFile, {
          path: args,
        });
        break;
      default:
        invoke(SeelenCommand.ActivateNotification, {
          umid: notification.appUmid,
          args,
          inputData: $data.value,
        });
    }
  }

  return (
    <motion.div
      animate={{ x: "0%", opacity: 1 }}
      exit={{ x: "100%", opacity: 0 }}
      initial={{ x: "100%", opacity: 1 }}
      transition={{ duration: 0.4 }}
    >
      <div
        className="notification"
        onClick={() => {
          if (notification.content["@launch"]) {
            onAction(
              notification.content["@launch"],
              notification.content["@activationType"],
            );
          }
        }}
      >
        <div className="notification-header">
          <div className="notification-header-info">
            <FileIcon
              className="notification-icon"
              umid={notification.appUmid}
            />
            <div>{notification.appName}</div>
            <span>-</span>
            <div>
              {moment(WindowsDateFileTimeToDate(notification.date)).fromNow()}
            </div>
          </div>
          <button
            className="notification-header-close"
            onClick={(e) => {
              e.stopPropagation();
              invoke(SeelenCommand.NotificationsClose, { id: notification.id });
              onClose?.();
            }}
          >
            <Icon iconName="IoClose" />
          </button>
        </div>

        <div className="notification-body">
          {logoImage && logoImage["@src"] && (
            <img
              src={logoImage["@src"]}
              alt={logoImage["@alt"] || ""}
              className={cx("notification-body-logo-image", {
                "notification-body-logo-image-circle": logoImage["@hint-crop"] === "Circle",
              })}
            />
          )}

          <div className="notification-body-content">
            {body.map((entry, index) => <NotificationBindingEntry key={index} entry={entry} />)}
          </div>

          {heroImage && heroImage["@src"] && (
            <img
              src={heroImage["@src"]}
              alt={heroImage["@alt"] || ""}
              className="notification-body-hero-image"
            />
          )}
        </div>

        {!!actions.length && (
          <div className="notification-actions">
            {actions.map((entry, index) => (
              <NotificationActionEntry
                key={index}
                entry={entry}
                inputsData={$data.value}
                onInputChange={onInputChange}
                onAction={onAction}
              />
            ))}
          </div>
        )}
      </div>
    </motion.div>
  );
}

function NotificationBindingEntry({ entry }: { entry: ToastBindingChild }) {
  if ("text" in entry) {
    return <p>{entry.text.$value}</p>;
  }

  if ("image" in entry && entry.image["@src"]) {
    // these placement are handled separately
    if (
      entry.image["@placement"] === "AppLogoOverride" ||
      entry.image["@placement"] === "Hero"
    ) {
      return null;
    }
    return <img src={entry.image["@src"]} alt={entry.image["@alt"] || ""} />;
  }

  if ("group" in entry) {
    return (
      <div className="notification-group">
        {entry.group.subgroup.map((subgroup, index) => (
          <div key={index} className="notification-subgroup">
            {subgroup.$value.map((entry, index) => <NotificationBindingEntry key={index} entry={entry} />)}
          </div>
        ))}
      </div>
    );
  }

  return null;
}

interface NotificationActionEntryProps {
  entry: ToastActionsChild;
  inputsData: Record<string, string>;
  onInputChange: (key: string, value: string) => void;
  onAction: (args: string, method: ToastActionActivationType) => void;
}

function NotificationActionEntry({
  entry,
  inputsData,
  onInputChange,
  onAction,
}: NotificationActionEntryProps) {
  if ("input" in entry) {
    const input = entry.input;
    switch (input["@type"]) {
      case "Text":
        return (
          <input
            className="notification-input"
            placeholder={input["@placeHolderContent"] || ""}
            value={inputsData[input["@id"]] || ""}
            onClick={(e) => {
              e.stopPropagation();
            }}
            onChange={(e) => {
              onInputChange(input["@id"], e.currentTarget.value);
            }}
          />
        );
      case "Selection":
        return (
          <Select
            size="small"
            placeholder={input["@placeHolderContent"]}
            value={inputsData[input["@id"]]}
            options={input.selection.map((opt) => ({
              value: opt["@id"],
              label: opt["@content"],
            }))}
            onClick={(e) => {
              e.stopPropagation();
            }}
            onSelect={(value) => {
              onInputChange(input["@id"], value);
            }}
          />
        );
    }
  }

  if ("action" in entry && entry.action["@placement"] !== "ContextMenu") {
    return (
      <Tooltip title={entry.action["@hint-toolTip"]}>
        <button
          className="notification-action"
          onClick={(e) => {
            e.stopPropagation();
            onAction(
              entry.action["@arguments"],
              entry.action["@activationType"],
            );
          }}
        >
          {entry.action["@content"]}
        </button>
      </Tooltip>
    );
  }

  return null;
}

function splitToastContent(notification: AppNotification) {
  const toast = notification.content;
  const template = toast.visual.binding["@template"];
  const actions = toast.actions?.$value || [];

  let logoImage: ToastImage | null = null;
  let heroImage: ToastImage | null = null;
  const body: ToastBindingChild[] = [];

  for (const entry of toast.visual.binding.$value) {
    if ("image" in entry) {
      if (
        entry.image["@placement"] === "AppLogoOverride" ||
        (!entry.image["@placement"] && !logoImage &&
          template.startsWith("ToastImageAndText"))
      ) {
        logoImage = entry.image;
        continue;
      }

      if (entry.image["@placement"] === "Hero") {
        heroImage = entry.image;
        continue;
      }
    }
    body.push(entry);
  }

  return { logoImage, heroImage, body, actions };
}

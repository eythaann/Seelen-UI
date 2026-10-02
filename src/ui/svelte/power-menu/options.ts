import { invoke } from "@tauri-apps/api/core";
import { SeelenCommand } from "@seelen-ui/lib";

interface Option {
  key: string;
  /** label used instead of `key` when pending updates will be installed */
  updateKey?: string;
  icon: string;
  onClick: (installUpdates: boolean) => void;
}

export const options: Option[] = [
  {
    key: "lock",
    icon: "IoLockClosed",
    onClick() {
      invoke(SeelenCommand.Lock);
    },
  },
  {
    key: "log_out",
    icon: "IoLogOutOutline",
    onClick() {
      invoke(SeelenCommand.LogOut);
    },
  },
  {
    key: "shutdown",
    updateKey: "update_and_shutdown",
    icon: "IoPower",
    onClick(installUpdates) {
      invoke(SeelenCommand.Shutdown, { installUpdates });
    },
  },
  {
    key: "reboot",
    updateKey: "update_and_reboot",
    icon: "MdRestartAlt",
    onClick(installUpdates) {
      invoke(SeelenCommand.Restart, { installUpdates });
    },
  },
  {
    key: "suspend",
    icon: "BiMoon",
    onClick() {
      invoke(SeelenCommand.Suspend);
    },
  },
  {
    key: "hibernate",
    icon: "TbZzz",
    onClick() {
      invoke(SeelenCommand.Hibernate);
    },
  },
];

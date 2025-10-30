import { invoke } from "@tauri-apps/api/core";
import { SeelenCommand } from "libs/core/npm/esm/mod";

interface Option {
  key: string;
  icon: string;
  onClick: () => void;
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
    icon: "IoPower",
    onClick() {
      invoke(SeelenCommand.Shutdown);
    },
  },
  {
    key: "reboot",
    icon: "MdRestartAlt",
    onClick() {
      invoke(SeelenCommand.Restart);
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

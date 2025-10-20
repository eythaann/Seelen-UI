interface Option {
  key: string;
  icon: string;
  onClick: () => void;
}

export const options: Option[] = [
  {
    key: "lock",
    icon: "IoLockClosed",
    onClick() {},
  },
  {
    key: "log_out",
    icon: "IoLogOutOutline",
    onClick() {},
  },
  {
    key: "shutdown",
    icon: "IoPower",
    onClick() {},
  },
  {
    key: "reboot",
    icon: "MdRestartAlt",
    onClick() {},
  },
  {
    key: "suspend",
    icon: "BiMoon",
    onClick() {},
  },
  {
    key: "hibernate",
    icon: "TbZzz",
    onClick() {},
  },
];

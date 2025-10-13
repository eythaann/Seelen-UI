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
    key: "logout",
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
    key: "sleep",
    icon: "BiMoon",
    onClick() {},
  },
  {
    key: "hibernate",
    icon: "TbZzz",
    onClick() {},
  },
];

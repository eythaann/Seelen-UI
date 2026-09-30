const GUIDS_TO_IGNORE = [
  "7820ae73-23e3-4229-82c1-e41cb67d5b9c", // speaker volument icon
  "7820ae74-23e3-4229-82c1-e41cb67d5b9c", // network icon
  "7820ae75-23e3-4229-82c1-e41cb67d5b9c", // battery icon
];

const items = trayIcons
  .filter((icon) => icon.isVisible && icon.isPromoted && !GUIDS_TO_IGNORE.includes(icon.guid))
  .map((icon) => {
    const onClick = `invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: "${icon.registryKey}",
        action: "LeftClick",
      })`;

    const onAuxClick = `invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: "${icon.registryKey}",
        action: "MiddleClick",
      })`;

    const onContextMenu = `invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: "${icon.registryKey}",
        action: "RightClick",
      })`;

    return Button({
      onClick,
      onAuxClick,
      onContextMenu,
      content: Image({
        path: icon.iconPath,
      }),
    });
  });

const OverflowButton = Button({
  content: Icon({ name: "IoIosArrowDropdown" }),
  onClick: `trigger("@seelen/system-tray");`,
});
items.push(OverflowButton);

if (self.placement === "left") {
  items.reverse();
}

return items;

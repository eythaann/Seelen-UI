const GUIDS_TO_IGNORE = [
  "7820ae73-23e3-4229-82c1-e41cb67d5b9c", // speaker volument icon
  "7820ae74-23e3-4229-82c1-e41cb67d5b9c", // network icon
  "7820ae75-23e3-4229-82c1-e41cb67d5b9c", // battery icon
];

const items = trayIcons
  .filter((icon) => icon.isVisible && icon.isPromoted && !GUIDS_TO_IGNORE.includes(icon.guid))
  .map((icon) => {
    const onClick = `invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: "${icon.sessionId}",
        action: "LeftClick",
      })`;

    const onDoubleClick = `invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: "${icon.sessionId}",
        action: "LeftDoubleClick",
      })`;

    const onAuxClick = `invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: "${icon.sessionId}",
        action: "MiddleClick",
      })`;

    const onContextMenu = `invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: "${icon.sessionId}",
        action: "RightClick",
      })`;

    return Button({
      tooltip: icon.tooltip,
      onClick,
      onDoubleClick,
      onAuxClick,
      onContextMenu,
      content: Image({
        path: icon.iconPath,
        version: icon.iconImageHash,
        // single color glyphs drawn for the taskbar theme, tinted by the theme instead
        isMask: icon.isGlyph,
      }),
    });
  });

const OverflowButton = Button({
  tooltip: "System Tray",
  content: Icon({ name: "IoIosArrowDropdown" }),
  onClick: `trigger("@seelen/system-tray");`,
});
items.push(OverflowButton);

if (self.placement === "left") {
  items.reverse();
}

return items;

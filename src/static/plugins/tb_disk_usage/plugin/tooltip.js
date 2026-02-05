const units = ["KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

return disks
  .map((disk) => {
    let total = disk.totalSpace;
    let used = disk.totalSpace - disk.availableSpace;
    let unit = "B";

    units.forEach((unitSize) => {
      if (total >= 1024) {
        total /= 1024;
        used /= 1024;
        unit = unitSize;
      } else {
        return;
      }
    });

    return disk.mountPoint + " -> " + used.toFixed(1) + unit + " / " + total.toFixed(1) + unit;
  })
  .join("\n");

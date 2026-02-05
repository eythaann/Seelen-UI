const units = ["KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

let totalRead = disks.reduce((total, disk) => total + disk.readBytes, 0);
let totalWritten = disks.reduce((total, disk) => total + disk.writtenBytes, 0);
let unit = "B";

units.forEach((unitSize) => {
  if (totalRead >= 1024 || totalWritten >= 1024) {
    totalRead /= 1024;
    totalWritten /= 1024;
    unit = unitSize;
  } else {
    return;
  }
});

return [
  icon("BsDeviceHdd"),
  " ",
  totalRead.toFixed(0) + unit + "/s" + " | " + totalWritten.toFixed(0) + unit + "/s",
];

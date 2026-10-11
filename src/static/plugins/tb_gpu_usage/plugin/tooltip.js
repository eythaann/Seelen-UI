const units = ["KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

function format(bytes) {
  let value = bytes;
  let unit = "B";
  for (const unitSize of units) {
    if (value < 1024) break;
    value /= 1024;
    unit = unitSize;
  }
  return value.toFixed(1) + unit;
}

return gpus
  .map((gpu) => {
    const memory = gpu.dedicatedMemoryTotal > 0
      ? format(gpu.dedicatedMemoryUsed) + " / " + format(gpu.dedicatedMemoryTotal)
      : format(gpu.sharedMemoryUsed) + " / " + format(gpu.sharedMemoryTotal);
    return gpu.name + " -> " + gpu.usage.toFixed(0) + "% | " + memory;
  })
  .join("\n");

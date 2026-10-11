// with multiple adapters (integrated + dedicated) show the busiest one
const used = gpus.reduce((max, gpu) => Math.max(max, gpu.usage), 0);

return [icon("BsGpuCard"), " ", used.toFixed(0) + "%"];

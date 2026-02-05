const totalUsage = cores.reduce((total, core) => total + core.usage, 0);
const used = totalUsage / cores.length;

return [icon("LuCpu"), " ", used.toFixed(0) + "%"];

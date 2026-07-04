const connectedDevices = devices.filter((d) => d.connected);

if (!connectedDevices.length) {
  return Icon({ name: "TbBluetooth" });
}

const deviceIcons = [];

for (let i = 0; i < connectedDevices.length; i++) {
  deviceIcons.push(Icon({ name: getIconNameForBTDevice(connectedDevices[i]) }));

  if (i !== connectedDevices.length - 1) {
    deviceIcons.push(" ");
  }
}

return [...deviceIcons, " ", Icon({ name: "TbBluetoothConnected" })];

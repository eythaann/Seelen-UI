import type {
  BLEAppearance,
  BLEAppearanceHumanInterfaceDeviceSubCategory,
  BluetoothAudioVideoMinor,
  BluetoothClass,
  BluetoothComputerMinor,
  BluetoothDevice,
  BluetoothHealthMinor,
  BluetoothImagingMinor,
  BluetoothLANNetworkAccessPointMinor,
  BluetoothPeripheralMinor,
  BluetoothPeripheralSubMinor,
  BluetoothPhoneMinor,
  BluetoothToyMinor,
  BluetoothWearableMinor,
} from "@seelen-ui/lib/types";

const UNKNOWN_ICON = "TbDeviceUnknown";

type IconsDict<T> = Record<Extract<T, string>, string>;

/** Returns `dict[value]`, falling back to `fallbackIcon` when `value` isn't a plain string (e.g. `{ Reserved: number }`). */
function iconFor<T extends string>(dict: Record<T, string>, value: T | unknown, fallbackIcon: string): string {
  return typeof value === "string" ? dict[value as T] : fallbackIcon;
}

const COMPUTER_ICONS: IconsDict<BluetoothComputerMinor> = {
  Uncategorized: "IoDesktopOutline",
  DesktopWorkstation: "IoDesktopOutline",
  ServerclassComputer: "IoServerOutline",
  Laptop: "IoLaptopOutline",
  HandheldPCPDA: "IoMdPhoneLandscape",
  PalmsizePCPDA: "IoMdPhoneLandscape",
  Tablet: "IoIosTabletPortrait",
  WearableComputer: "IoMdWatch",
};

const PHONE_ICONS: IconsDict<BluetoothPhoneMinor> = {
  Uncategorized: "IoPhonePortraitOutline",
  Cellular: "IoPhonePortraitOutline",
  Cordless: "IoPhonePortraitOutline",
  Smartphone: "IoPhonePortraitOutline",
  WiredModemorVoiceGateway: "BsModem",
  CommonISDNAccess: "LuPhone",
};

const LAN_NETWORK_ICONS: IconsDict<BluetoothLANNetworkAccessPointMinor> = {
  Fullyavailable: "PiNetwork",
  N1to17utilized: "PiNetwork",
  N17to33utilized: "PiNetwork",
  N33to50utilized: "PiNetwork",
  N50to67utilized: "PiNetwork",
  N67to83utilized: "PiNetwork",
  N83to99utilized: "PiNetwork",
  Noserviceavailable: "PiNetworkX",
};

const AUDIO_VIDEO_ICONS: IconsDict<BluetoothAudioVideoMinor> = {
  Uncategorized: "LuSpeaker",
  WearableHeadsetDevice: "IoHeadset",
  HandsfreeDevice: "IoHeadset",
  ReservedforFutureUse: "LuSpeaker",
  Microphone: "HiOutlineMicrophone",
  Loudspeaker: "LuSpeaker",
  Headphones: "IoHeadset",
  PortableAudio: "LuSpeaker",
  CarAudio: "BsPciCardSound",
  Settopbox: "CgModem",
  HiFiAudioDevice: "IoHeadset",
  VCR: "TbCapProjecting",
  VideoCamera: "HiOutlineVideoCamera",
  Camcorder: "HiOutlineVideoCamera",
  VideoMonitor: "PiMonitorPlay",
  VideoDisplayandLoudspeaker: "PiMonitorPlay",
  VideoConferencing: "PiVideoConference",
  ReservedforFutureUse0x11: "LuSpeaker",
  GamingToy: "GiGameConsole",
  HearingAid: "IoEarOutline",
  Glasses: "IoGlassesOutline",
};

const PERIPHERAL_SUBMINOR_ICONS: IconsDict<BluetoothPeripheralSubMinor> = {
  Uncategorized: "IoGameControllerOutline",
  Joystick: "LuJoystick",
  Gamepad: "IoGameControllerOutline",
  RemoteControl: "RiRemoteControl2Line",
  SensingDevice: "MdSensorOccupied",
  DigitizerTablet: "IoTabletLandscapeOutline",
  CardReader: "MdOutlineChromeReaderMode",
  DigitalPen: "IoPencil",
  HandheldScanner: "MdOutlineScanner",
  HandheldGesturalInputDevice: "MdOutlineGesture",
};

const PERIPHERAL_MINOR_ICONS: IconsDict<BluetoothPeripheralMinor> = {
  Uncategorized: PERIPHERAL_SUBMINOR_ICONS.Uncategorized,
  Keyboard: "BsKeyboard",
  PointingDevice: "BsMouse",
  ComboKeyboardPointingDevice: "BsKeyboard",
};

const WEARABLE_ICONS: IconsDict<BluetoothWearableMinor> = {
  Wristwatch: "IoWatchOutline",
  Pager: "FaPager",
  Jacket: "TbJacket",
  Helmet: "GiFullMotorcycleHelmet",
  Glasses: "IoGlassesOutline",
  Pin: "IoMdPricetag",
};

const TOY_ICONS: IconsDict<BluetoothToyMinor> = {
  Robot: "RiRobot3Line",
  Vehicle: "FaCar",
  DollActionFigure: "LiaBabySolid",
  Controller: "IoGameControllerOutline",
  Game: "CgGames",
};

const HEALTH_ICONS: IconsDict<BluetoothHealthMinor> = {
  Undefined: "MdOutlineMedication",
  BloodPressureMonitor: "MdOutlineBloodtype",
  Thermometer: "IoIosThermometer",
  WeighingScale: "FaWeightScale",
  GlucoseMeter: "PiSpeedometerBold",
  PulseOximeter: "BsClipboardPulse",
  HeartPulseRateMonitor: "BsHeartPulse",
  HealthDataDisplay: "TbHeartRateMonitor",
  StepCounter: "IoFootstepsSharp",
  BodyCompositionAnalyzer: "IoBodyOutline",
  PeakFlowMonitor: "TbHeartRateMonitor",
  MedicationMonitor: "MdOutlineMedication",
  KneeProsthesis: "GiRobotLeg",
  AnkleProsthesis: "GiMechanicalArm",
  GenericHealthManager: "MdOutlineMedication",
  PersonalMobilityDevice: "FaWheelchair",
};

function iconForClass(cls: BluetoothClass): string {
  switch (cls.major) {
    case "Miscellaneous":
    case "Uncategorized":
    case "Reserved":
      return UNKNOWN_ICON;
    case "Computer":
      return iconFor(COMPUTER_ICONS, cls.minor, COMPUTER_ICONS.Uncategorized);
    case "Phone":
      return iconFor(PHONE_ICONS, cls.minor, PHONE_ICONS.Uncategorized);
    case "LANNetworkAccessPoint":
      return iconFor(LAN_NETWORK_ICONS, cls.minor, LAN_NETWORK_ICONS.Fullyavailable);
    case "AudioVideo":
      return iconFor(AUDIO_VIDEO_ICONS, cls.minor, AUDIO_VIDEO_ICONS.Uncategorized);
    case "Peripheral": {
      if (typeof cls.subminor === "string" && cls.subminor !== "Uncategorized") {
        return PERIPHERAL_SUBMINOR_ICONS[cls.subminor];
      }
      return iconFor(PERIPHERAL_MINOR_ICONS, cls.minor, PERIPHERAL_MINOR_ICONS.Uncategorized);
    }
    case "Imaging": {
      const flags: BluetoothImagingMinor[] = cls.minor;
      if (flags.includes("Display")) return "BsDisplay";
      if (flags.includes("Scanner") || flags.includes("Printer")) {
        return "IoPrintOutline";
      }
      if (flags.includes("Camera")) return "IoCameraOutline";
      return "IoImagesOutline";
    }
    case "Wearable":
      return iconFor(WEARABLE_ICONS, cls.minor, WEARABLE_ICONS.Wristwatch);
    case "Toy":
      return iconFor(TOY_ICONS, cls.minor, "LuToyBrick");
    case "Health":
      return iconFor(HEALTH_ICONS, cls.minor, HEALTH_ICONS.Undefined);
  }
}

const HUMAN_INTERFACE_DEVICE_ICONS: IconsDict<
  BLEAppearanceHumanInterfaceDeviceSubCategory
> = {
  Keyboard: "BsKeyboard",
  Mouse: "BsMouse",
  Joystick: "LuJoystick",
  Gamepad: "IoGameControllerOutline",
  DigitizerTablet: "IoTabletLandscapeOutline",
  CardReader: "MdOutlineChromeReaderMode",
  DigitalPen: "IoPencil",
  BarcodeScanner: "MdBarcodeReader",
  Touchpad: "LuTouchpad",
  PresentationRemote: "PiVideoConference",
};

type FuncByAppearance = {
  [key in BLEAppearance["category"]]:
    | string
    | ((
      subcategory: Extract<BLEAppearance, { category: key }>["subcategory"],
    ) => string);
};

const APPEARANCE_ICONS: FuncByAppearance = {
  Unknown: UNKNOWN_ICON,
  Phone: "IoPhonePortraitOutline",
  Computer: "IoDesktopOutline",
  Watch: "IoWatchOutline",
  Clock: "IoTimeOutline",
  Display: "BsDisplay",
  RemoteControl: "RiRemoteControl2Line",
  Eyeglasses: "IoGlassesOutline",
  Tag: "IoMdPricetag",
  Keyring: "IoMdKey",
  MediaPlayer: "PiMonitorPlay",
  BarcodeScanner: "MdOutlineScanner",
  Thermometer: "IoIosThermometer",
  HeartRateSensor: "BsHeartPulse",
  BloodPressure: "MdOutlineBloodtype",
  HumanInterfaceDevice: (sub) => {
    if (typeof sub !== "string") return "BsKeyboard";
    return HUMAN_INTERFACE_DEVICE_ICONS[sub];
  },
  GlucoseMeter: "PiSpeedometerBold",
  RunningWalkingSensor: "IoFootstepsSharp",
  Cycling: "MdDirectionsBike",
  ControlDevice: "RiRemoteControl2Line",
  NetworkDevice: "PiNetwork",
  Sensor: "MdSensorOccupied",
  LightFixtures: "IoMdBulb",
  Fan: "BsFan",
  HVAC: "MdAcUnit",
  AirConditioning: "MdAcUnit",
  Humidifier: "TbDroplet",
  Heating: "MdElectricBolt",
  AccessControl: "MdLock",
  MotorizedDevice: "MdOutlinePrecisionManufacturing",
  PowerDevice: "MdPower",
  LightSource: "IoMdBulb",
  WindowCovering: "MdBlinds",
  AudioSink: "LuSpeaker",
  AudioSource: "HiOutlineMicrophone",
  MotorizedVehicle: "FaCar",
  DomesticAppliance: "MdMicrowave",
  WearableAudioDevice: "IoHeadset",
  Aircraft: "FaPlane",
  AVEquipment: "PiMonitorPlay",
  DisplayEquipment: "BsDisplay",
  Hearingaid: "IoEarOutline",
  Gaming: "IoGameControllerOutline",
  Signage: "MdSignpost",
  PulseOximeter: "BsClipboardPulse",
  WeightScale: "FaWeightScale",
  PersonalMobilityDevice: "FaWheelchair",
  ContinuousGlucoseMonitor: "PiSpeedometerBold",
  InsulinPump: "MdOutlineMedication",
  MedicationDelivery: "MdOutlineMedication",
  Spirometer: "GiThermometerScale",
  OutdoorSportsActivity: "MdSportsBaseball",
  IndustrialMeasurementDevice: "MdOutlinePrecisionManufacturing",
  IndustrialTools: "MdBuild",
};

export function getIconByAppearance(appearance: BLEAppearance): string {
  const icon = APPEARANCE_ICONS[appearance.category];
  if (typeof icon === "string") {
    return icon;
  }
  return icon(appearance.subcategory as any);
}

export function getIconNameForBTDevice(device: BluetoothDevice): string {
  if (device.appearance) {
    return getIconByAppearance(device.appearance);
  }
  return iconForClass(device.class);
}

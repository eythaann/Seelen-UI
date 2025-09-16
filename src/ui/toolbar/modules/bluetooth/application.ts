import {
  BLEAppearance,
  BLEAppearanceHumanInterfaceDeviceSubCategory,
  BluetoothAudioVideoMinor,
  BluetoothComputerMinor,
  BluetoothDevice,
  BluetoothHealthMinor,
  BluetoothMinorClass,
  BluetoothNetworkMinor,
  BluetoothPeripheralMinor,
  BluetoothPeripheralSubMinor,
  BluetoothPhoneMinor,
  BluetoothToyMinor,
  BluetoothWearableMinor,
} from "@seelen-ui/lib/types";
import { IconName } from "@shared/components/Icon/icons";
import { prettify, unionToIntersection } from "readable-types";

const UNKNOWN_ICON = "TbDeviceUnknown";

type IconsDict<T> = prettify<Record<Extract<T, string>, IconName>>;

const COMPUTER_ICONS: IconsDict<BluetoothComputerMinor> = {
  Uncategorized: "IoDesktopOutline",
  Desktop: "IoDesktopOutline",
  Server: "IoServerOutline",
  Laptop: "IoLaptopOutline",
  Handheld: "IoMdPhoneLandscape",
  PalmSize: "IoMdPhoneLandscape",
  Tablet: "IoIosTabletPortrait",
  Wearable: "IoMdWatch",
};

const PHONE_ICONS: IconsDict<BluetoothPhoneMinor> = {
  Uncategorized: "IoPhonePortraitOutline",
  Cellular: "IoPhonePortraitOutline",
  Cordless: "IoPhonePortraitOutline",
  SmartPhone: "IoPhonePortraitOutline",
  Wired: "BsModem",
  Isdn: "LuPhone",
};

const NETWORK_ICONS: IconsDict<BluetoothNetworkMinor> = {
  FullyAvailable: "PiNetwork",
  Used01To17Percent: "PiNetwork",
  Used17To33Percent: "PiNetwork",
  Used33To50Percent: "PiNetwork",
  Used50To67Percent: "PiNetwork",
  Used67To83Percent: "PiNetwork",
  Used83To99Percent: "PiNetwork",
  NoServiceAvailable: "PiNetworkX",
};

const AUDIO_VIDEO_ICONS: IconsDict<BluetoothAudioVideoMinor> = {
  Uncategorized: "LuSpeaker",
  Headset: "IoHeadset",
  HandsFree: "IoHeadset",
  Microphone: "HiOutlineMicrophone",
  Loudspeaker: "LuSpeaker",
  Headphones: "IoHeadset",
  PortableAudio: "LuSpeaker",
  CarAudio: "BsPciCardSound",
  SetTopBox: "CgModem",
  HiFiAudioDevice: "IoHeadset",
  Vcr: "TbCapProjecting",
  VideoCamera: "HiOutlineVideoCamera",
  CamCorder: "HiOutlineVideoCamera",
  VideoMonitor: "PiMonitorPlay",
  VideoDisplayAndLoudspeaker: "PiMonitorPlay",
  VideoConferencing: "PiVideoConference",
  GamingToy: "GiGameConsole",
};

const PERIPHERAL_SUBMINOR_ICONS: IconsDict<BluetoothPeripheralSubMinor> = {
  Uncategorized: "IoGameControllerOutline",
  Joystick: "LuJoystick",
  Gamepad: "IoGameControllerOutline",
  RemoteControl: "RiRemoteControl2Line",
  Sensor: "MdSensorOccupied",
  DigitizerTablet: "IoTabletLandscapeOutline",
  CardReader: "MdOutlineChromeReaderMode",
  DigitalPen: "IoPencil",
  HandheldScanner: "MdOutlineScanner",
  HandheldGestural: "MdOutlineGesture",
};

const PERIPHERAL_MINOR_ICONS: IconsDict<BluetoothPeripheralMinor> = {
  Uncategorized: PERIPHERAL_SUBMINOR_ICONS.Uncategorized,
  Keyboard: "BsKeyboard",
  Pointing: "BsMouse",
  ComboKeyboardPointing: "BsKeyboard",
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
  Doll: "LiaBabySolid",
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
  HeartPulseMonitor: "BsHeartPulse",
  HealthDataDisplay: "TbHeartRateMonitor",
  StepCounter: "IoFootstepsSharp",
  BodyCompositionMonitor: "IoBodyOutline",
  PeakFlowMonitor: "TbHeartRateMonitor",
  MedicationMonitor: "MdOutlineMedication",
  KneeProsthesis: "GiRobotLeg",
  AnkleProsthesis: "GiMechanicalArm",
  GenericHealthManager: "MdOutlineMedication",
  PersonalMobilityDevice: "FaWheelchair",
};

type AllMinors = unionToIntersection<BluetoothMinorClass>;
type FuncByMajor = {
  [key in keyof AllMinors]: (minor: AllMinors[key]) => IconName;
};

const FUNC_BY_MAJOR: FuncByMajor = {
  Miscellaneous: () => UNKNOWN_ICON,
  Computer: (minor) => {
    if (typeof minor !== "string") return COMPUTER_ICONS.Uncategorized;
    return COMPUTER_ICONS[minor];
  },
  Phone: (minor) => {
    if (typeof minor !== "string") return PHONE_ICONS.Uncategorized;
    return PHONE_ICONS[minor];
  },
  NetworkAccessPoint: ([minor, _subminor]) => {
    return NETWORK_ICONS[minor];
  },
  AudioVideo: (minor) => {
    if (typeof minor !== "string") return AUDIO_VIDEO_ICONS.Uncategorized;
    return AUDIO_VIDEO_ICONS[minor];
  },
  Peripheral: ([minor, subminor]) => {
    if (typeof subminor === "string" && subminor !== "Uncategorized") {
      return PERIPHERAL_SUBMINOR_ICONS[subminor];
    }
    return PERIPHERAL_MINOR_ICONS[minor];
  },
  Imaging: ([minors, _subminor]) => {
    if (minors.includes("Display")) return "BsDisplay";
    if (minors.includes("Scanner") || minors.includes("Printer")) {
      return "IoPrintOutline";
    }
    if (minors.includes("Camera")) return "IoCameraOutline";
    return "IoImagesOutline";
  },
  Wearable: (minor) => {
    if (typeof minor !== "string") return WEARABLE_ICONS.Wristwatch;
    return WEARABLE_ICONS[minor];
  },
  Toy: (minor) => {
    if (typeof minor !== "string") return "LuToyBrick";
    return TOY_ICONS[minor];
  },
  Health: (minor) => {
    if (typeof minor !== "string") return HEALTH_ICONS.Undefined;
    return HEALTH_ICONS[minor];
  },
  Uncategorized: () => UNKNOWN_ICON,
};

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
    | IconName
    | ((
      subcategory: Extract<BLEAppearance, { category: key }>["subcategory"],
    ) => IconName);
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

export function getIconByAppearance(appearance: BLEAppearance): IconName {
  const icon = APPEARANCE_ICONS[appearance.category];
  if (typeof icon === "string") {
    return icon;
  }
  return icon(appearance.subcategory as any);
}

export function getIconForBTDevice(device: BluetoothDevice): IconName {
  if (device.appearance) {
    return getIconByAppearance(device.appearance);
  }

  const Minor = device.minorClass as unionToIntersection<BluetoothMinorClass>;
  const Major = Object.keys(Minor)[0]!;
  const func = FUNC_BY_MAJOR[Major];
  return func(Minor[Major] as any);
}

export function getMinorAsString(minor: BluetoothMinorClass): string {
  const MinorObj = minor as unionToIntersection<BluetoothMinorClass>;
  const Major = Object.keys(MinorObj)[0]!;
  const Minor = MinorObj[Major];
  if (typeof Minor === "string") {
    return Minor;
  }
  return JSON.stringify(Minor);
}

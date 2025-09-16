// This file was generated via rust macros. Don't modify manually.
// all this structs are based on official docs https://www.bluetooth.com/specifications/assigned-numbers

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceUnknownSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearancePhoneSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceComputerSubCategory {
    DesktopWorkstation = 0x1,
    ServerclassComputer = 0x2,
    Laptop = 0x3,
    HandheldPCPDAclamshell = 0x4,
    PalmsizePCPDA = 0x5,
    Wearablecomputerwatchsize = 0x6,
    Tablet = 0x7,
    DockingStation = 0x8,
    AllinOne = 0x9,
    BladeServer = 0xa,
    Convertible = 0xb,
    Detachable = 0xc,
    IoTGateway = 0xd,
    MiniPC = 0xe,
    StickPC = 0xf,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceWatchSubCategory {
    SportsWatch = 0x1,
    Smartwatch = 0x2,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceClockSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceDisplaySubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceRemoteControlSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceEyeglassesSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceTagSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceKeyringSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceMediaPlayerSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceBarcodeScannerSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceThermometerSubCategory {
    EarThermometer = 0x1,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceHeartRateSensorSubCategory {
    HeartRateBelt = 0x1,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceBloodPressureSubCategory {
    ArmBloodPressure = 0x1,
    WristBloodPressure = 0x2,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceHumanInterfaceDeviceSubCategory {
    Keyboard = 0x1,
    Mouse = 0x2,
    Joystick = 0x3,
    Gamepad = 0x4,
    DigitizerTablet = 0x5,
    CardReader = 0x6,
    DigitalPen = 0x7,
    BarcodeScanner = 0x8,
    Touchpad = 0x9,
    PresentationRemote = 0xa,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceGlucoseMeterSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceRunningWalkingSensorSubCategory {
    InShoeRunningWalkingSensor = 0x1,
    OnShoeRunningWalkingSensor = 0x2,
    OnHipRunningWalkingSensor = 0x3,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceCyclingSubCategory {
    CyclingComputer = 0x1,
    SpeedSensor = 0x2,
    CadenceSensor = 0x3,
    PowerSensor = 0x4,
    SpeedandCadenceSensor = 0x5,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceControlDeviceSubCategory {
    Switch = 0x1,
    Multiswitch = 0x2,
    Button = 0x3,
    Slider = 0x4,
    RotarySwitch = 0x5,
    TouchPanel = 0x6,
    SingleSwitch = 0x7,
    DoubleSwitch = 0x8,
    TripleSwitch = 0x9,
    BatterySwitch = 0xa,
    EnergyHarvestingSwitch = 0xb,
    PushButton = 0xc,
    Dial = 0xd,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceNetworkDeviceSubCategory {
    AccessPoint = 0x1,
    MeshDevice = 0x2,
    MeshNetworkProxy = 0x3,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceSensorSubCategory {
    MotionSensor = 0x1,
    AirqualitySensor = 0x2,
    TemperatureSensor = 0x3,
    HumiditySensor = 0x4,
    LeakSensor = 0x5,
    SmokeSensor = 0x6,
    OccupancySensor = 0x7,
    ContactSensor = 0x8,
    CarbonMonoxideSensor = 0x9,
    CarbonDioxideSensor = 0xa,
    AmbientLightSensor = 0xb,
    EnergySensor = 0xc,
    ColorLightSensor = 0xd,
    RainSensor = 0xe,
    FireSensor = 0xf,
    WindSensor = 0x10,
    ProximitySensor = 0x11,
    MultiSensor = 0x12,
    FlushMountedSensor = 0x13,
    CeilingMountedSensor = 0x14,
    WallMountedSensor = 0x15,
    Multisensor = 0x16,
    EnergyMeter = 0x17,
    FlameDetector = 0x18,
    VehicleTirePressureSensor = 0x19,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceLightFixturesSubCategory {
    WallLight = 0x1,
    CeilingLight = 0x2,
    FloorLight = 0x3,
    CabinetLight = 0x4,
    DeskLight = 0x5,
    TrofferLight = 0x6,
    PendantLight = 0x7,
    IngroundLight = 0x8,
    FloodLight = 0x9,
    UnderwaterLight = 0xa,
    BollardwithLight = 0xb,
    PathwayLight = 0xc,
    GardenLight = 0xd,
    PoletopLight = 0xe,
    Spotlight = 0xf,
    LinearLight = 0x10,
    StreetLight = 0x11,
    ShelvesLight = 0x12,
    BayLight = 0x13,
    EmergencyExitLight = 0x14,
    LightController = 0x15,
    LightDriver = 0x16,
    Bulb = 0x17,
    LowbayLight = 0x18,
    HighbayLight = 0x19,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceFanSubCategory {
    CeilingFan = 0x1,
    AxialFan = 0x2,
    ExhaustFan = 0x3,
    PedestalFan = 0x4,
    DeskFan = 0x5,
    WallFan = 0x6,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceHVACSubCategory {
    Thermostat = 0x1,
    Humidifier = 0x2,
    Dehumidifier = 0x3,
    Heater = 0x4,
    Radiator = 0x5,
    Boiler = 0x6,
    HeatPump = 0x7,
    InfraredHeater = 0x8,
    RadiantPanelHeater = 0x9,
    FanHeater = 0xa,
    AirCurtain = 0xb,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceAirConditioningSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceHumidifierSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceHeatingSubCategory {
    Radiator = 0x1,
    Boiler = 0x2,
    HeatPump = 0x3,
    InfraredHeater = 0x4,
    RadiantPanelHeater = 0x5,
    FanHeater = 0x6,
    AirCurtain = 0x7,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceAccessControlSubCategory {
    AccessDoor = 0x1,
    GarageDoor = 0x2,
    EmergencyExitDoor = 0x3,
    AccessLock = 0x4,
    Elevator = 0x5,
    Window = 0x6,
    EntranceGate = 0x7,
    DoorLock = 0x8,
    Locker = 0x9,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceMotorizedDeviceSubCategory {
    MotorizedGate = 0x1,
    Awning = 0x2,
    BlindsorShades = 0x3,
    Curtains = 0x4,
    Screen = 0x5,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearancePowerDeviceSubCategory {
    PowerOutlet = 0x1,
    PowerStrip = 0x2,
    Plug = 0x3,
    PowerSupply = 0x4,
    LEDDriver = 0x5,
    FluorescentLampGear = 0x6,
    HIDLampGear = 0x7,
    ChargeCase = 0x8,
    PowerBank = 0x9,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceLightSourceSubCategory {
    IncandescentLightBulb = 0x1,
    LEDLamp = 0x2,
    HIDLamp = 0x3,
    FluorescentLamp = 0x4,
    LEDArray = 0x5,
    MultiColorLEDArray = 0x6,
    Lowvoltagehalogen = 0x7,
    OrganiclightemittingdiodeOLED = 0x8,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceWindowCoveringSubCategory {
    WindowShades = 0x1,
    WindowBlinds = 0x2,
    WindowAwning = 0x3,
    WindowCurtain = 0x4,
    ExteriorShutter = 0x5,
    ExteriorScreen = 0x6,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceAudioSinkSubCategory {
    StandaloneSpeaker = 0x1,
    Soundbar = 0x2,
    BookshelfSpeaker = 0x3,
    StandmountedSpeaker = 0x4,
    Speakerphone = 0x5,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceAudioSourceSubCategory {
    Microphone = 0x1,
    Alarm = 0x2,
    Bell = 0x3,
    Horn = 0x4,
    BroadcastingDevice = 0x5,
    ServiceDesk = 0x6,
    Kiosk = 0x7,
    BroadcastingRoom = 0x8,
    Auditorium = 0x9,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceMotorizedVehicleSubCategory {
    Car = 0x1,
    LargeGoodsVehicle = 0x2,
    Vehicle2Wheels = 0x3,
    Motorbike = 0x4,
    Scooter = 0x5,
    Moped = 0x6,
    Vehicle3Wheels = 0x7,
    LightVehicle = 0x8,
    QuadBike = 0x9,
    Minibus = 0xa,
    Bus = 0xb,
    Trolley = 0xc,
    AgriculturalVehicle = 0xd,
    CamperCaravan = 0xe,
    RecreationalVehicleMotorHome = 0xf,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceDomesticApplianceSubCategory {
    Refrigerator = 0x1,
    Freezer = 0x2,
    Oven = 0x3,
    Microwave = 0x4,
    Toaster = 0x5,
    WashingMachine = 0x6,
    Dryer = 0x7,
    Coffeemaker = 0x8,
    Clothesiron = 0x9,
    Curlingiron = 0xa,
    Hairdryer = 0xb,
    Vacuumcleaner = 0xc,
    Roboticvacuumcleaner = 0xd,
    Ricecooker = 0xe,
    Clothessteamer = 0xf,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceWearableAudioDeviceSubCategory {
    Earbud = 0x1,
    Headset = 0x2,
    Headphones = 0x3,
    NeckBand = 0x4,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceAircraftSubCategory {
    LightAircraft = 0x1,
    Microlight = 0x2,
    Paraglider = 0x3,
    LargePassengerAircraft = 0x4,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceAVEquipmentSubCategory {
    Amplifier = 0x1,
    Receiver = 0x2,
    Radio = 0x3,
    Tuner = 0x4,
    Turntable = 0x5,
    CDPlayer = 0x6,
    DVDPlayer = 0x7,
    BlurayPlayer = 0x8,
    OpticalDiscPlayer = 0x9,
    SetTopBox = 0xa,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceDisplayEquipmentSubCategory {
    Television = 0x1,
    Monitor = 0x2,
    Projector = 0x3,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceHearingaidSubCategory {
    Inearhearingaid = 0x1,
    Behindearhearingaid = 0x2,
    CochlearImplant = 0x3,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceGamingSubCategory {
    HomeVideoGameConsole = 0x1,
    Portablehandheldconsole = 0x2,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceSignageSubCategory {
    DigitalSignage = 0x1,
    ElectronicLabel = 0x2,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearancePulseOximeterSubCategory {
    FingertipPulseOximeter = 0x1,
    WristWornPulseOximeter = 0x2,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceWeightScaleSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearancePersonalMobilityDeviceSubCategory {
    PoweredWheelchair = 0x1,
    MobilityScooter = 0x2,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceContinuousGlucoseMonitorSubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceInsulinPumpSubCategory {
    InsulinPumpdurablepump = 0x1,
    InsulinPumppatchpump = 0x4,
    InsulinPen = 0x8,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceMedicationDeliverySubCategory {
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceSpirometerSubCategory {
    HandheldSpirometer = 0x1,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceOutdoorSportsActivitySubCategory {
    LocationDisplay = 0x1,
    LocationandNavigationDisplay = 0x2,
    LocationPod = 0x3,
    LocationandNavigationPod = 0x4,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceIndustrialMeasurementDeviceSubCategory {
    TorqueTestingDevice = 0x1,
    Caliper = 0x2,
    DialIndicator = 0x3,
    Micrometer = 0x4,
    HeightGauge = 0x5,
    ForceGauge = 0x6,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(export_to = "BLEAppearanceSubCategory.ts")]
#[repr(u16)]
pub enum BLEAppearanceIndustrialToolsSubCategory {
    MachineToolHolder = 0x1,
    GenericClampingDevice = 0x2,
    ClampingJawsJawChuck = 0x3,
    ClampingColletChuck = 0x4,
    ClampingMandrel = 0x5,
    Vise = 0x6,
    ZeroPointClampingSystem = 0x7,
    TorqueWrench = 0x8,
    TorqueScrewdriver = 0x9,
    #[num_enum(catch_all)]
    Reserved(u16),
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[repr(u16)]
pub enum BLEAppearanceCategory {
    #[default]
    Unknown = 0x0,
    Phone = 0x1,
    Computer = 0x2,
    Watch = 0x3,
    Clock = 0x4,
    Display = 0x5,
    RemoteControl = 0x6,
    Eyeglasses = 0x7,
    Tag = 0x8,
    Keyring = 0x9,
    MediaPlayer = 0xa,
    BarcodeScanner = 0xb,
    Thermometer = 0xc,
    HeartRateSensor = 0xd,
    BloodPressure = 0xe,
    HumanInterfaceDevice = 0xf,
    GlucoseMeter = 0x10,
    RunningWalkingSensor = 0x11,
    Cycling = 0x12,
    ControlDevice = 0x13,
    NetworkDevice = 0x14,
    Sensor = 0x15,
    LightFixtures = 0x16,
    Fan = 0x17,
    HVAC = 0x18,
    AirConditioning = 0x19,
    Humidifier = 0x1a,
    Heating = 0x1b,
    AccessControl = 0x1c,
    MotorizedDevice = 0x1d,
    PowerDevice = 0x1e,
    LightSource = 0x1f,
    WindowCovering = 0x20,
    AudioSink = 0x21,
    AudioSource = 0x22,
    MotorizedVehicle = 0x23,
    DomesticAppliance = 0x24,
    WearableAudioDevice = 0x25,
    Aircraft = 0x26,
    AVEquipment = 0x27,
    DisplayEquipment = 0x28,
    Hearingaid = 0x29,
    Gaming = 0x2a,
    Signage = 0x2b,
    PulseOximeter = 0x31,
    WeightScale = 0x32,
    PersonalMobilityDevice = 0x33,
    ContinuousGlucoseMonitor = 0x34,
    InsulinPump = 0x35,
    MedicationDelivery = 0x36,
    Spirometer = 0x37,
    OutdoorSportsActivity = 0x51,
    IndustrialMeasurementDevice = 0x52,
    IndustrialTools = 0x53,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "category", content = "subcategory")]
pub enum BLEAppearance {
    Unknown(BLEAppearanceUnknownSubCategory),
    Phone(BLEAppearancePhoneSubCategory),
    Computer(BLEAppearanceComputerSubCategory),
    Watch(BLEAppearanceWatchSubCategory),
    Clock(BLEAppearanceClockSubCategory),
    Display(BLEAppearanceDisplaySubCategory),
    RemoteControl(BLEAppearanceRemoteControlSubCategory),
    Eyeglasses(BLEAppearanceEyeglassesSubCategory),
    Tag(BLEAppearanceTagSubCategory),
    Keyring(BLEAppearanceKeyringSubCategory),
    MediaPlayer(BLEAppearanceMediaPlayerSubCategory),
    BarcodeScanner(BLEAppearanceBarcodeScannerSubCategory),
    Thermometer(BLEAppearanceThermometerSubCategory),
    HeartRateSensor(BLEAppearanceHeartRateSensorSubCategory),
    BloodPressure(BLEAppearanceBloodPressureSubCategory),
    HumanInterfaceDevice(BLEAppearanceHumanInterfaceDeviceSubCategory),
    GlucoseMeter(BLEAppearanceGlucoseMeterSubCategory),
    RunningWalkingSensor(BLEAppearanceRunningWalkingSensorSubCategory),
    Cycling(BLEAppearanceCyclingSubCategory),
    ControlDevice(BLEAppearanceControlDeviceSubCategory),
    NetworkDevice(BLEAppearanceNetworkDeviceSubCategory),
    Sensor(BLEAppearanceSensorSubCategory),
    LightFixtures(BLEAppearanceLightFixturesSubCategory),
    Fan(BLEAppearanceFanSubCategory),
    HVAC(BLEAppearanceHVACSubCategory),
    AirConditioning(BLEAppearanceAirConditioningSubCategory),
    Humidifier(BLEAppearanceHumidifierSubCategory),
    Heating(BLEAppearanceHeatingSubCategory),
    AccessControl(BLEAppearanceAccessControlSubCategory),
    MotorizedDevice(BLEAppearanceMotorizedDeviceSubCategory),
    PowerDevice(BLEAppearancePowerDeviceSubCategory),
    LightSource(BLEAppearanceLightSourceSubCategory),
    WindowCovering(BLEAppearanceWindowCoveringSubCategory),
    AudioSink(BLEAppearanceAudioSinkSubCategory),
    AudioSource(BLEAppearanceAudioSourceSubCategory),
    MotorizedVehicle(BLEAppearanceMotorizedVehicleSubCategory),
    DomesticAppliance(BLEAppearanceDomesticApplianceSubCategory),
    WearableAudioDevice(BLEAppearanceWearableAudioDeviceSubCategory),
    Aircraft(BLEAppearanceAircraftSubCategory),
    AVEquipment(BLEAppearanceAVEquipmentSubCategory),
    DisplayEquipment(BLEAppearanceDisplayEquipmentSubCategory),
    Hearingaid(BLEAppearanceHearingaidSubCategory),
    Gaming(BLEAppearanceGamingSubCategory),
    Signage(BLEAppearanceSignageSubCategory),
    PulseOximeter(BLEAppearancePulseOximeterSubCategory),
    WeightScale(BLEAppearanceWeightScaleSubCategory),
    PersonalMobilityDevice(BLEAppearancePersonalMobilityDeviceSubCategory),
    ContinuousGlucoseMonitor(BLEAppearanceContinuousGlucoseMonitorSubCategory),
    InsulinPump(BLEAppearanceInsulinPumpSubCategory),
    MedicationDelivery(BLEAppearanceMedicationDeliverySubCategory),
    Spirometer(BLEAppearanceSpirometerSubCategory),
    OutdoorSportsActivity(BLEAppearanceOutdoorSportsActivitySubCategory),
    IndustrialMeasurementDevice(BLEAppearanceIndustrialMeasurementDeviceSubCategory),
    IndustrialTools(BLEAppearanceIndustrialToolsSubCategory),
}

impl From<u16> for BLEAppearance {
    fn from(value: u16) -> Self {
        let category = BLEAppearanceCategory::from(value >> 6); // 10 bits
        let subcategory = value & 0b111111; // 6 bits

        match category {
            BLEAppearanceCategory::Unknown => {
                BLEAppearance::Unknown(BLEAppearanceUnknownSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Phone => {
                BLEAppearance::Phone(BLEAppearancePhoneSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Computer => {
                BLEAppearance::Computer(BLEAppearanceComputerSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Watch => {
                BLEAppearance::Watch(BLEAppearanceWatchSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Clock => {
                BLEAppearance::Clock(BLEAppearanceClockSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Display => {
                BLEAppearance::Display(BLEAppearanceDisplaySubCategory::from(subcategory))
            }
            BLEAppearanceCategory::RemoteControl => BLEAppearance::RemoteControl(
                BLEAppearanceRemoteControlSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Eyeglasses => {
                BLEAppearance::Eyeglasses(BLEAppearanceEyeglassesSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Tag => {
                BLEAppearance::Tag(BLEAppearanceTagSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Keyring => {
                BLEAppearance::Keyring(BLEAppearanceKeyringSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::MediaPlayer => {
                BLEAppearance::MediaPlayer(BLEAppearanceMediaPlayerSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::BarcodeScanner => BLEAppearance::BarcodeScanner(
                BLEAppearanceBarcodeScannerSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Thermometer => {
                BLEAppearance::Thermometer(BLEAppearanceThermometerSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::HeartRateSensor => BLEAppearance::HeartRateSensor(
                BLEAppearanceHeartRateSensorSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::BloodPressure => BLEAppearance::BloodPressure(
                BLEAppearanceBloodPressureSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::HumanInterfaceDevice => BLEAppearance::HumanInterfaceDevice(
                BLEAppearanceHumanInterfaceDeviceSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::GlucoseMeter => {
                BLEAppearance::GlucoseMeter(BLEAppearanceGlucoseMeterSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::RunningWalkingSensor => BLEAppearance::RunningWalkingSensor(
                BLEAppearanceRunningWalkingSensorSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Cycling => {
                BLEAppearance::Cycling(BLEAppearanceCyclingSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::ControlDevice => BLEAppearance::ControlDevice(
                BLEAppearanceControlDeviceSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::NetworkDevice => BLEAppearance::NetworkDevice(
                BLEAppearanceNetworkDeviceSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Sensor => {
                BLEAppearance::Sensor(BLEAppearanceSensorSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::LightFixtures => BLEAppearance::LightFixtures(
                BLEAppearanceLightFixturesSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Fan => {
                BLEAppearance::Fan(BLEAppearanceFanSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::HVAC => {
                BLEAppearance::HVAC(BLEAppearanceHVACSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::AirConditioning => BLEAppearance::AirConditioning(
                BLEAppearanceAirConditioningSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Humidifier => {
                BLEAppearance::Humidifier(BLEAppearanceHumidifierSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Heating => {
                BLEAppearance::Heating(BLEAppearanceHeatingSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::AccessControl => BLEAppearance::AccessControl(
                BLEAppearanceAccessControlSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::MotorizedDevice => BLEAppearance::MotorizedDevice(
                BLEAppearanceMotorizedDeviceSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::PowerDevice => {
                BLEAppearance::PowerDevice(BLEAppearancePowerDeviceSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::LightSource => {
                BLEAppearance::LightSource(BLEAppearanceLightSourceSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::WindowCovering => BLEAppearance::WindowCovering(
                BLEAppearanceWindowCoveringSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::AudioSink => {
                BLEAppearance::AudioSink(BLEAppearanceAudioSinkSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::AudioSource => {
                BLEAppearance::AudioSource(BLEAppearanceAudioSourceSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::MotorizedVehicle => BLEAppearance::MotorizedVehicle(
                BLEAppearanceMotorizedVehicleSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::DomesticAppliance => BLEAppearance::DomesticAppliance(
                BLEAppearanceDomesticApplianceSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::WearableAudioDevice => BLEAppearance::WearableAudioDevice(
                BLEAppearanceWearableAudioDeviceSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Aircraft => {
                BLEAppearance::Aircraft(BLEAppearanceAircraftSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::AVEquipment => {
                BLEAppearance::AVEquipment(BLEAppearanceAVEquipmentSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::DisplayEquipment => BLEAppearance::DisplayEquipment(
                BLEAppearanceDisplayEquipmentSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Hearingaid => {
                BLEAppearance::Hearingaid(BLEAppearanceHearingaidSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Gaming => {
                BLEAppearance::Gaming(BLEAppearanceGamingSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::Signage => {
                BLEAppearance::Signage(BLEAppearanceSignageSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::PulseOximeter => BLEAppearance::PulseOximeter(
                BLEAppearancePulseOximeterSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::WeightScale => {
                BLEAppearance::WeightScale(BLEAppearanceWeightScaleSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::PersonalMobilityDevice => BLEAppearance::PersonalMobilityDevice(
                BLEAppearancePersonalMobilityDeviceSubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::ContinuousGlucoseMonitor => {
                BLEAppearance::ContinuousGlucoseMonitor(
                    BLEAppearanceContinuousGlucoseMonitorSubCategory::from(subcategory),
                )
            }
            BLEAppearanceCategory::InsulinPump => {
                BLEAppearance::InsulinPump(BLEAppearanceInsulinPumpSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::MedicationDelivery => BLEAppearance::MedicationDelivery(
                BLEAppearanceMedicationDeliverySubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::Spirometer => {
                BLEAppearance::Spirometer(BLEAppearanceSpirometerSubCategory::from(subcategory))
            }
            BLEAppearanceCategory::OutdoorSportsActivity => BLEAppearance::OutdoorSportsActivity(
                BLEAppearanceOutdoorSportsActivitySubCategory::from(subcategory),
            ),
            BLEAppearanceCategory::IndustrialMeasurementDevice => {
                BLEAppearance::IndustrialMeasurementDevice(
                    BLEAppearanceIndustrialMeasurementDeviceSubCategory::from(subcategory),
                )
            }
            BLEAppearanceCategory::IndustrialTools => BLEAppearance::IndustrialTools(
                BLEAppearanceIndustrialToolsSubCategory::from(subcategory),
            ),
        }
    }
}

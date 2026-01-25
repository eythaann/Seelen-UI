#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum BluetoothMajorServiceClass {
    LimitedDiscoverableMode = 0x1,
    LowEnergyAudio = 0x2,
    Reserved = 0x4,
    /// Location identification
    Positioning = 0x8,
    /// LAN, Ad hoc, ...
    Networking = 0x10,
    /// Printing, Speakers, ...
    Rendering = 0x20,
    /// Scanner, Microphone, ...
    Capturing = 0x40,
    /// v-Inbox, v-Folder, ...
    ObjectTransfer = 0x80,
    /// Speaker, Microphone, Headset service, ...
    Audio = 0x100,
    /// Cordless telephony, Modem, Headset service, ...
    Telephony = 0x200,
    /// WEB-server, WAP-server, ...
    Information = 0x400,
}

/// 5 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(repr(enum = name))]
pub enum BluetoothMajorClass {
    /// The ”Miscellaneous” Major Device Class is used where a more specific
    /// Major Device Class code is not suitable.\
    /// A device that does not have a Major Class Code assigned can use the
    /// ”Uncategorized: device code not specified” code until classified.
    Miscellaneous = 0x0,
    /// Computer (desktop, notebook, PDA, organizer, ...)
    Computer = 0x1,
    /// Phone (cellular, cordless, payphone, modem, ...)
    Phone = 0x2,
    /// LAN/NetworkAccessPoint
    NetworkAccessPoint = 0x3,
    /// Audio/Video (headset, speaker, stereo, video display, VCR, ...)
    AudioVideo = 0x4,
    /// Peripheral (mouse, joystick, keyboard, ...)
    Peripheral = 0x5,
    /// Imaging (printer, scanner, camera, display, ...)
    Imaging = 0x6,
    Wearable = 0x7,
    Toy = 0x8,
    Health = 0x9,
    /// Device code not specified
    #[default]
    Uncategorized = 0x1F,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, TS)]
pub enum BluetoothMinorClass {
    Miscellaneous { unused: u8 },
    Computer(BluetoothComputerMinor),
    Phone(BluetoothPhoneMinor),
    NetworkAccessPoint(BluetoothNetworkMinor, BluetoothNetworkSubMinor),
    AudioVideo(BluetoothAudioVideoMinor),
    Peripheral(BluetoothPeripheralMinor, BluetoothPeripheralSubMinor),
    Imaging(Vec<BluetoothImagingMinor>, BluetoothImagingSubMinor),
    Wearable(BluetoothWearableMinor),
    Toy(BluetoothToyMinor),
    Health(BluetoothHealthMinor),
    Uncategorized { unused: u8 },
}

/// 6 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothComputerMinor {
    /// Code for device not specified
    Uncategorized = 0x0,
    /// Desktop Workstation, PC
    Desktop = 0x1,
    /// Server-class Computer
    Server = 0x2,
    Laptop = 0x3,
    /// Handheld PC/PDA (e.g. clamshell)
    Handheld = 0x4,
    /// Palm-size PC/PDA
    PalmSize = 0x5,
    /// Wearable computer (e.g. watch)
    Wearable = 0x6,
    Tablet = 0x7,
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// 6 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothPhoneMinor {
    /// Code for device not specified
    Uncategorized = 0x0,
    Cellular = 0x1,
    Cordless = 0x2,
    SmartPhone = 0x3,
    /// Wired Modem or Voice Gateway
    Wired = 0x4,
    /// Common ISDN access device
    Isdn = 0x5,
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// 3 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(repr(enum = name))]
pub enum BluetoothNetworkMinor {
    FullyAvailable = 0x0,
    Used01To17Percent = 0x1,
    Used17To33Percent = 0x2,
    Used33To50Percent = 0x3,
    Used50To67Percent = 0x4,
    Used67To83Percent = 0x5,
    Used83To99Percent = 0x6,
    #[default]
    NoServiceAvailable = 0x7,
}

/// 3 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothNetworkSubMinor {
    Uncategorized = 0x0,
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// 6 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothAudioVideoMinor {
    /// Code for device not specified
    Uncategorized = 0x0,
    /// Wearable Headset
    Headset = 0x1,
    /// Hands-free Headset
    HandsFree = 0x2,
    Microphone = 0x4, // 0x3 is reserved
    Loudspeaker = 0x5,
    Headphones = 0x6,
    PortableAudio = 0x7,
    CarAudio = 0x8,
    /// Set-top box
    SetTopBox = 0x9,
    HiFiAudioDevice = 0xA,
    Vcr = 0xB,
    VideoCamera = 0xC,
    CamCorder = 0xD,
    VideoMonitor = 0xE,
    VideoDisplayAndLoudspeaker = 0xF,
    VideoConferencing = 0x10,
    GamingToy = 0x12, // 0x11 is reserved
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// 2 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
#[ts(repr(enum = name))]
pub enum BluetoothPeripheralMinor {
    /// Code for device not specified
    #[default]
    Uncategorized = 0x0,
    Keyboard = 0x1,
    Pointing = 0x2,
    ComboKeyboardPointing = 0x3,
}

/// 4 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothPeripheralSubMinor {
    Uncategorized = 0x0,
    Joystick = 0x1,
    Gamepad = 0x2,
    RemoteControl = 0x3,
    Sensor = 0x4,
    DigitizerTablet = 0x5,
    CardReader = 0x6,
    DigitalPen = 0x7,
    /// Handheld scanner (e.g. barcodes, RFID)
    HandheldScanner = 0x8,
    /// Handheld Gestural Input Device (e.g., “wand” form factor)
    HandheldGestural = 0x9,
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// 4 bits, flags that can be combined
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum BluetoothImagingMinor {
    Display = 0x1,
    Camera = 0x2,
    Scanner = 0x4,
    Printer = 0x8,
}

/// 2 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothImagingSubMinor {
    Uncategorized = 0x0,
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// 6 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothWearableMinor {
    Wristwatch = 0x1,
    Pager = 0x2,
    Jacket = 0x3,
    Helmet = 0x4,
    Glasses = 0x5,
    /// Pin (e.g., lapel pin, broach, badge)
    Pin = 0x6,
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// 6 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothToyMinor {
    Robot = 0x1,
    Vehicle = 0x2,
    /// Doll/Action Figure
    Doll = 0x3,
    Controller = 0x4,
    Game = 0x5,
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// 6 bits
#[repr(u8)]
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS,
)]
pub enum BluetoothHealthMinor {
    Undefined = 0x0,
    BloodPressureMonitor = 0x1,
    Thermometer = 0x2,
    WeighingScale = 0x3,
    GlucoseMeter = 0x4,
    PulseOximeter = 0x5,
    HeartPulseMonitor = 0x6,
    HealthDataDisplay = 0x7,
    StepCounter = 0x8,
    BodyCompositionMonitor = 0x9,
    PeakFlowMonitor = 0xA,
    MedicationMonitor = 0xB,
    KneeProsthesis = 0xC,
    AnkleProsthesis = 0xD,
    GenericHealthManager = 0xE,
    PersonalMobilityDevice = 0xF,
    #[num_enum(catch_all)]
    Reserved(u8),
}

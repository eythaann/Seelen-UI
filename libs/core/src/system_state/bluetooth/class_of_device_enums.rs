// This file was generated via rust macros. Don't modify manually.
// all this structs are based on official docs https://bitbucket.org/bluetooth-SIG/public/src/main/assigned_numbers/core/class_of_device.yaml

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, TS)]
#[repr(u32)]
pub enum BluetoothMajorServiceClass {
    LimitedDiscoverableMode = 0x1,
    LEaudio = 0x2,
    ReservedforFutureUse = 0x4,
    Positioning = 0x8,
    Networking = 0x10,
    Rendering = 0x20,
    Capturing = 0x40,
    ObjectTransfer = 0x80,
    Audio = 0x100,
    Telephony = 0x200,
    Information = 0x400,
}

impl BluetoothMajorServiceClass {
    /// `bits` should already be shifted so the lowest declared service bit is bit 0
    pub fn from_bits(bits: u32) -> Vec<Self> {
        use BluetoothMajorServiceClass::*;
        [LimitedDiscoverableMode, LEaudio, ReservedforFutureUse, Positioning, Networking, Rendering, Capturing, ObjectTransfer, Audio, Telephony, Information]
            .into_iter()
            .filter(|&service| bits & service as u32 != 0)
            .collect()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothComputerMinor {
    Uncategorized = 0x0,
    DesktopWorkstation = 0x1,
    ServerclassComputer = 0x2,
    Laptop = 0x3,
    HandheldPCPDA = 0x4,
    PalmsizePCPDA = 0x5,
    WearableComputer = 0x6,
    Tablet = 0x7,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothPhoneMinor {
    Uncategorized = 0x0,
    Cellular = 0x1,
    Cordless = 0x2,
    Smartphone = 0x3,
    WiredModemorVoiceGateway = 0x4,
    CommonISDNAccess = 0x5,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothLANNetworkAccessPointMinor {
    Fullyavailable = 0x0,
    N1to17utilized = 0x1,
    N17to33utilized = 0x2,
    N33to50utilized = 0x3,
    N50to67utilized = 0x4,
    N67to83utilized = 0x5,
    N83to99utilized = 0x6,
    Noserviceavailable = 0x7,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothLANNetworkAccessPointSubMinor {
    Uncategorized = 0x0,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothAudioVideoMinor {
    Uncategorized = 0x0,
    WearableHeadsetDevice = 0x1,
    HandsfreeDevice = 0x2,
    ReservedforFutureUse = 0x3,
    Microphone = 0x4,
    Loudspeaker = 0x5,
    Headphones = 0x6,
    PortableAudio = 0x7,
    CarAudio = 0x8,
    Settopbox = 0x9,
    HiFiAudioDevice = 0xa,
    VCR = 0xb,
    VideoCamera = 0xc,
    Camcorder = 0xd,
    VideoMonitor = 0xe,
    VideoDisplayandLoudspeaker = 0xf,
    VideoConferencing = 0x10,
    ReservedforFutureUse0x11 = 0x11,
    GamingToy = 0x12,
    HearingAid = 0x13,
    Glasses = 0x14,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothPeripheralMinor {
    Uncategorized = 0x0,
    Keyboard = 0x1,
    PointingDevice = 0x2,
    ComboKeyboardPointingDevice = 0x3,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothPeripheralSubMinor {
    Uncategorized = 0x0,
    Joystick = 0x1,
    Gamepad = 0x2,
    RemoteControl = 0x3,
    SensingDevice = 0x4,
    DigitizerTablet = 0x5,
    CardReader = 0x6,
    DigitalPen = 0x7,
    HandheldScanner = 0x8,
    HandheldGesturalInputDevice = 0x9,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothImagingMinor {
    Display = 0x1,
    Camera = 0x2,
    Scanner = 0x4,
    Printer = 0x8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothImagingSubMinor {
    Uncategorized = 0x0,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothWearableMinor {
    Wristwatch = 0x1,
    Pager = 0x2,
    Jacket = 0x3,
    Helmet = 0x4,
    Glasses = 0x5,
    Pin = 0x6,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothToyMinor {
    Robot = 0x1,
    Vehicle = 0x2,
    DollActionFigure = 0x3,
    Controller = 0x4,
    Game = 0x5,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]
#[repr(u8)]
pub enum BluetoothHealthMinor {
    Undefined = 0x0,
    BloodPressureMonitor = 0x1,
    Thermometer = 0x2,
    WeighingScale = 0x3,
    GlucoseMeter = 0x4,
    PulseOximeter = 0x5,
    HeartPulseRateMonitor = 0x6,
    HealthDataDisplay = 0x7,
    StepCounter = 0x8,
    BodyCompositionAnalyzer = 0x9,
    PeakFlowMonitor = 0xa,
    MedicationMonitor = 0xb,
    KneeProsthesis = 0xc,
    AnkleProsthesis = 0xd,
    GenericHealthManager = 0xe,
    PersonalMobilityDevice = 0xf,
    #[num_enum(catch_all)]
    Reserved(u8),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "major")]
pub enum BluetoothClass {
    Miscellaneous { minor: u8 },
    Computer { minor: BluetoothComputerMinor },
    Phone { minor: BluetoothPhoneMinor },
    LANNetworkAccessPoint { minor: BluetoothLANNetworkAccessPointMinor, subminor: BluetoothLANNetworkAccessPointSubMinor },
    AudioVideo { minor: BluetoothAudioVideoMinor },
    Peripheral { minor: BluetoothPeripheralMinor, subminor: BluetoothPeripheralSubMinor },
    Imaging { minor: Vec<BluetoothImagingMinor>, subminor: BluetoothImagingSubMinor },
    Wearable { minor: BluetoothWearableMinor },
    Toy { minor: BluetoothToyMinor },
    Health { minor: BluetoothHealthMinor },
    Uncategorized { minor: u8 },
    Reserved { minor: u8 },
}

impl BluetoothClass {
    /// `major` must already be masked down to its 5 bits and `minor` to its 6 bits
    pub fn from_major_and_minor(major: u8, minor: u8) -> Self {
        match major {
            0x0 => BluetoothClass::Miscellaneous { minor },
            0x1 => BluetoothClass::Computer { minor: BluetoothComputerMinor::from(minor) },
            0x2 => BluetoothClass::Phone { minor: BluetoothPhoneMinor::from(minor) },
            0x3 => {
                let upper = (minor >> 3) & 0x7;
                let lower = minor & 0x7;
                BluetoothClass::LANNetworkAccessPoint { minor: BluetoothLANNetworkAccessPointMinor::from(upper), subminor: BluetoothLANNetworkAccessPointSubMinor::from(lower) }
            }
            0x4 => BluetoothClass::AudioVideo { minor: BluetoothAudioVideoMinor::from(minor) },
            0x5 => {
                let upper = (minor >> 4) & 0x3;
                let lower = minor & 0xf;
                BluetoothClass::Peripheral { minor: BluetoothPeripheralMinor::from(upper), subminor: BluetoothPeripheralSubMinor::from(lower) }
            }
            0x6 => {
                let upper = (minor >> 2) & 0xf;
                let lower = minor & 0x3;
                use BluetoothImagingMinor::*;
                let flags = [Display, Camera, Scanner, Printer]
                    .into_iter()
                    .filter(|&flag| upper & flag as u8 != 0)
                    .collect();
                BluetoothClass::Imaging { minor: flags, subminor: BluetoothImagingSubMinor::from(lower) }
            }
            0x7 => BluetoothClass::Wearable { minor: BluetoothWearableMinor::from(minor) },
            0x8 => BluetoothClass::Toy { minor: BluetoothToyMinor::from(minor) },
            0x9 => BluetoothClass::Health { minor: BluetoothHealthMinor::from(minor) },
            0x1f => BluetoothClass::Uncategorized { minor },
            _ => BluetoothClass::Reserved { minor },
        }
    }
}
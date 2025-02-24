use std::path::PathBuf;

use windows::Devices::Bluetooth::{
    BluetoothConnectionStatus, BluetoothDevice, BluetoothLEDevice, BluetoothMajorClass,
    BluetoothMinorClass,
};

use crate::{
    log_error, modules::bluetooth::BLUETOOTH_MANAGER, trace_lock, windows_api::WindowsApi,
};

use seelen_core::system_state::{
    BluetoothDevice as VMDevice, BluetoothMajor as VMMajor, BluetoothMinor as VMMinor,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDeviceInfo {
    pub id: String,
    pub name: String,
    pub address: u64,
    pub major_class: BluetoothMajor,
    pub minor_main_class: BluetoothMinor,
    pub minor_sub_class: BluetoothMinor,
    pub connected: bool,
    pub paired: bool,
    pub can_be_paired: bool,
    pub icon_path: Option<PathBuf>,

    pub inner: Option<BluetoothDevice>,
    pub inner_le: Option<BluetoothLEDevice>,
}

impl From<BluetoothDeviceInfo> for VMDevice {
    fn from(val: BluetoothDeviceInfo) -> Self {
        VMDevice {
            id: val.id,
            name: val.name,
            address: val.address,
            major_class: val.major_class.into(),
            minor_main_class: val.minor_main_class.into(),
            minor_sub_class: val.minor_sub_class.into(),
            connected: val.connected,
            can_pair: val.can_be_paired,
            paired: val.paired,
            is_bluetooth_loweenergy: val.inner_le.is_some(),
            icon_path: val.icon_path,
        }
    }
}

impl TryFrom<BluetoothDevice> for BluetoothDeviceInfo {
    type Error = windows_core::Error;

    fn try_from(bluetooth_device: BluetoothDevice) -> windows_core::Result<Self> {
        let class = bluetooth_device.ClassOfDevice()?;
        let major: BluetoothMajor = class.MajorClass()?.into();
        let minor = class.MinorClass()?;
        let pairing_state = bluetooth_device.DeviceInformation()?.Pairing()?;

        Ok(Self {
            id: bluetooth_device.BluetoothDeviceId()?.Id()?.to_string(),
            name: bluetooth_device.Name()?.to_string(),
            address: bluetooth_device.BluetoothAddress()?,
            major_class: major.clone(),
            minor_main_class: BluetoothMinor::parse_main(minor, major.clone()),
            minor_sub_class: BluetoothMinor::parse_sub(minor, major),
            connected: bluetooth_device.ConnectionStatus()? == BluetoothConnectionStatus::Connected,
            paired: pairing_state.IsPaired()?,
            icon_path: WindowsApi::extract_thumbnail_from_stream(
                bluetooth_device
                    .DeviceInformation()?
                    .GetGlyphThumbnailAsync()?
                    .get()?
                    .into(),
            )
            .ok(),
            can_be_paired: pairing_state.CanPair()?,
            inner: Some(bluetooth_device),
            inner_le: None,
        })
    }
}
impl TryFrom<BluetoothLEDevice> for BluetoothDeviceInfo {
    type Error = windows_core::Error;

    fn try_from(bluetooth_device: BluetoothLEDevice) -> windows_core::Result<Self> {
        let pairing_state = bluetooth_device.DeviceInformation()?.Pairing()?;

        Ok(Self {
            id: bluetooth_device.BluetoothDeviceId()?.Id()?.to_string(),
            name: bluetooth_device.Name()?.to_string(),
            address: bluetooth_device.BluetoothAddress()?,
            major_class: BluetoothMajor::Unkown,
            minor_main_class: BluetoothMinor::Uncategorized,
            minor_sub_class: BluetoothMinor::Uncategorized,
            connected: bluetooth_device.ConnectionStatus()? == BluetoothConnectionStatus::Connected,
            paired: pairing_state.IsPaired()?,
            icon_path: WindowsApi::extract_thumbnail_from_stream(
                bluetooth_device
                    .DeviceInformation()?
                    .GetGlyphThumbnailAsync()?
                    .get()?
                    .into(),
            )
            .ok(),
            can_be_paired: pairing_state.CanPair()?,
            inner: None,
            inner_le: Some(bluetooth_device),
        })
    }
}

//Proxy event handlers for device attrivute changed
impl BluetoothDeviceInfo {
    pub(super) fn on_device_attribute_changed(
        sender: &Option<BluetoothDevice>,
        _args: &Option<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        if let Some(device) = sender {
            let mut manager = trace_lock!(BLUETOOTH_MANAGER);
            log_error!(manager.update_device(device.clone().try_into()?));
        }

        Ok(())
    }
    pub(super) fn on_le_device_attribute_changed(
        sender: &Option<BluetoothLEDevice>,
        _args: &Option<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        if let Some(device) = sender {
            let mut manager = trace_lock!(BLUETOOTH_MANAGER);
            log_error!(manager.update_device(device.clone().try_into()?));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothMajor {
    Miscellaneous,
    Computer,
    Phone,
    NetworkAccessPoint,
    AudioVideo,
    Peripheral,
    Imaging,
    Wearable,
    Toy,
    Health,
    Unkown,
}

impl From<BluetoothMajor> for VMMajor {
    fn from(val: BluetoothMajor) -> Self {
        unsafe { std::mem::transmute(val as u8) }
    }
}

impl From<BluetoothMajorClass> for BluetoothMajor {
    fn from(device_major: BluetoothMajorClass) -> Self {
        match device_major {
            BluetoothMajorClass::Miscellaneous => BluetoothMajor::Miscellaneous,
            BluetoothMajorClass::Computer => BluetoothMajor::Computer,
            BluetoothMajorClass::Phone => BluetoothMajor::Phone,
            BluetoothMajorClass::NetworkAccessPoint => BluetoothMajor::NetworkAccessPoint,
            BluetoothMajorClass::AudioVideo => BluetoothMajor::AudioVideo,
            BluetoothMajorClass::Peripheral => BluetoothMajor::Peripheral,
            BluetoothMajorClass::Imaging => BluetoothMajor::Imaging,
            BluetoothMajorClass::Wearable => BluetoothMajor::Wearable,
            BluetoothMajorClass::Toy => BluetoothMajor::Toy,
            BluetoothMajorClass::Health => BluetoothMajor::Health,
            _ => BluetoothMajor::Unkown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothMinor {
    Uncategorized,
    ComputerDesktop,
    ComputerServer,
    ComputerLaptop,
    ComputerHandheld,
    ComputerPalmSize,
    ComputerWearable,
    ComputerTablet,
    PhoneCellular,
    PhoneCordless,
    PhoneSmartPhone,
    PhoneWired,
    PhoneIsdn,
    NetworkFullyAvailable,
    NetworkUsed01To17Percent,
    NetworkUsed17To33Percent,
    NetworkUsed33To50Percent,
    NetworkUsed50To67Percent,
    NetworkUsed67To83Percent,
    NetworkUsed83To99Percent,
    NetworkNoServiceAvailable,
    AudioVideoWearableHeadset,
    AudioVideoHandsFree,
    AudioVideoMicrophone,
    AudioVideoLoudspeaker,
    AudioVideoHeadphones,
    AudioVideoPortableAudio,
    AudioVideoCarAudio,
    AudioVideoSetTopBox,
    AudioVideoHifiAudioDevice,
    AudioVideoVcr,
    AudioVideoVideoCamera,
    AudioVideoCamcorder,
    AudioVideoVideoMonitor,
    AudioVideoVideoDisplayAndLoudspeaker,
    AudioVideoVideoConferencing,
    AudioVideoGamingOrToy,
    PeripheralJoystick,
    PeripheralGamepad,
    PeripheralRemoteControl,
    PeripheralSensing,
    PeripheralDigitizerTablet,
    PeripheralCardReader,
    PeripheralDigitalPen,
    PeripheralHandheldScanner,
    PeripheralHandheldGesture,
    WearableWristwatch,
    WearablePager,
    WearableJacket,
    WearableHelmet,
    WearableGlasses,
    ToyRobot,
    ToyVehicle,
    ToyDoll,
    ToyController,
    ToyGame,
    HealthBloodPressureMonitor,
    HealthThermometer,
    HealthWeighingScale,
    HealthGlucoseMeter,
    HealthPulseOximeter,
    HealthHeartRateMonitor,
    HealthHealthDataDisplay,
    HealthStepCounter,
    HealthBodyCompositionAnalyzer,
    HealthPeakFlowMonitor,
    HealthMedicationMonitor,
    HealthKneeProsthesis,
    HealthAnkleProsthesis,
    HealthGenericHealthManager,
    HealthPersonalMobilityDevice,

    //Added because they were not identified by the windows api developer, but exists
    PeripheralOther,
    PeripheralPointer,
    PeripheralKeyboard,
    PeripheralKeyboardAndPointer,
}

impl From<BluetoothMinor> for VMMinor {
    fn from(val: BluetoothMinor) -> Self {
        unsafe { std::mem::transmute(val as u8) }
    }
}

// https://www.ampedrftech.com/datasheets/cod_definition.pdf ->  Major Device Class
impl BluetoothMinor {
    pub fn parse_sub(minor: BluetoothMinorClass, major: BluetoothMajor) -> Self {
        match major {
            BluetoothMajor::Peripheral => {
                let post = BluetoothMinorClass(minor.0 & 0b1111); //lower 4 bits are for other functional items
                match post {
                    BluetoothMinorClass::PeripheralJoystick => BluetoothMinor::PeripheralJoystick,
                    BluetoothMinorClass::PeripheralGamepad => BluetoothMinor::PeripheralGamepad,
                    BluetoothMinorClass::PeripheralRemoteControl => {
                        BluetoothMinor::PeripheralRemoteControl
                    }
                    BluetoothMinorClass::PeripheralSensing => BluetoothMinor::PeripheralSensing,
                    BluetoothMinorClass::PeripheralDigitizerTablet => {
                        BluetoothMinor::PeripheralDigitizerTablet
                    }
                    BluetoothMinorClass::PeripheralCardReader => {
                        BluetoothMinor::PeripheralCardReader
                    }
                    BluetoothMinorClass::PeripheralDigitalPen => {
                        BluetoothMinor::PeripheralDigitalPen
                    }
                    BluetoothMinorClass::PeripheralHandheldScanner => {
                        BluetoothMinor::PeripheralHandheldScanner
                    }
                    BluetoothMinorClass::PeripheralHandheldGesture => {
                        BluetoothMinor::PeripheralHandheldGesture
                    }
                    _ => BluetoothMinor::Uncategorized,
                }
            }
            _ => BluetoothMinor::Uncategorized,
        }
    }
    pub fn parse_main(minor: BluetoothMinorClass, major: BluetoothMajor) -> Self {
        match major {
            BluetoothMajor::Miscellaneous => BluetoothMinor::Uncategorized,
            BluetoothMajor::Computer => match minor {
                BluetoothMinorClass::ComputerDesktop => BluetoothMinor::ComputerDesktop,
                BluetoothMinorClass::ComputerServer => BluetoothMinor::ComputerServer,
                BluetoothMinorClass::ComputerLaptop => BluetoothMinor::ComputerLaptop,
                BluetoothMinorClass::ComputerHandheld => BluetoothMinor::ComputerHandheld,
                BluetoothMinorClass::ComputerPalmSize => BluetoothMinor::ComputerPalmSize,
                BluetoothMinorClass::ComputerWearable => BluetoothMinor::ComputerWearable,
                BluetoothMinorClass::ComputerTablet => BluetoothMinor::ComputerTablet,
                _ => BluetoothMinor::Uncategorized,
            },
            BluetoothMajor::Phone => match minor {
                BluetoothMinorClass::PhoneCellular => BluetoothMinor::PhoneCellular,
                BluetoothMinorClass::PhoneCordless => BluetoothMinor::PhoneCordless,
                BluetoothMinorClass::PhoneSmartPhone => BluetoothMinor::PhoneSmartPhone,
                BluetoothMinorClass::PhoneWired => BluetoothMinor::PhoneWired,
                BluetoothMinorClass::PhoneIsdn => BluetoothMinor::PhoneIsdn,
                _ => BluetoothMinor::Uncategorized,
            },
            BluetoothMajor::NetworkAccessPoint => match minor {
                BluetoothMinorClass::NetworkFullyAvailable => BluetoothMinor::NetworkFullyAvailable,
                BluetoothMinorClass::NetworkUsed01To17Percent => {
                    BluetoothMinor::NetworkUsed01To17Percent
                }
                BluetoothMinorClass::NetworkUsed17To33Percent => {
                    BluetoothMinor::NetworkUsed17To33Percent
                }
                BluetoothMinorClass::NetworkUsed33To50Percent => {
                    BluetoothMinor::NetworkUsed33To50Percent
                }
                BluetoothMinorClass::NetworkUsed50To67Percent => {
                    BluetoothMinor::NetworkUsed50To67Percent
                }
                BluetoothMinorClass::NetworkUsed67To83Percent => {
                    BluetoothMinor::NetworkUsed67To83Percent
                }
                BluetoothMinorClass::NetworkUsed83To99Percent => {
                    BluetoothMinor::NetworkUsed83To99Percent
                }
                BluetoothMinorClass::NetworkNoServiceAvailable => {
                    BluetoothMinor::NetworkNoServiceAvailable
                }
                _ => BluetoothMinor::Uncategorized,
            },
            BluetoothMajor::AudioVideo => match minor {
                BluetoothMinorClass::AudioVideoWearableHeadset => {
                    BluetoothMinor::AudioVideoWearableHeadset
                }
                BluetoothMinorClass::AudioVideoHandsFree => BluetoothMinor::AudioVideoHandsFree,
                BluetoothMinorClass::AudioVideoMicrophone => BluetoothMinor::AudioVideoMicrophone,
                BluetoothMinorClass::AudioVideoLoudspeaker => BluetoothMinor::AudioVideoLoudspeaker,
                BluetoothMinorClass::AudioVideoHeadphones => BluetoothMinor::AudioVideoHeadphones,
                BluetoothMinorClass::AudioVideoPortableAudio => {
                    BluetoothMinor::AudioVideoPortableAudio
                }
                BluetoothMinorClass::AudioVideoCarAudio => BluetoothMinor::AudioVideoCarAudio,
                BluetoothMinorClass::AudioVideoSetTopBox => BluetoothMinor::AudioVideoSetTopBox,
                BluetoothMinorClass::AudioVideoHifiAudioDevice => {
                    BluetoothMinor::AudioVideoHifiAudioDevice
                }
                BluetoothMinorClass::AudioVideoVcr => BluetoothMinor::AudioVideoVcr,
                BluetoothMinorClass::AudioVideoVideoCamera => BluetoothMinor::AudioVideoVideoCamera,
                BluetoothMinorClass::AudioVideoCamcorder => BluetoothMinor::AudioVideoCamcorder,
                BluetoothMinorClass::AudioVideoVideoMonitor => {
                    BluetoothMinor::AudioVideoVideoMonitor
                }
                BluetoothMinorClass::AudioVideoVideoDisplayAndLoudspeaker => {
                    BluetoothMinor::AudioVideoVideoDisplayAndLoudspeaker
                }
                BluetoothMinorClass::AudioVideoVideoConferencing => {
                    BluetoothMinor::AudioVideoVideoConferencing
                }
                BluetoothMinorClass::AudioVideoGamingOrToy => BluetoothMinor::AudioVideoGamingOrToy,
                _ => BluetoothMinor::Uncategorized,
            },
            BluetoothMajor::Peripheral => {
                let pre = minor.0 >> 4 & 0b11; //upper 2 bits are for keyboard and mouse
                match pre {
                    0 => BluetoothMinor::PeripheralOther,
                    1 => BluetoothMinor::PeripheralPointer,
                    2 => BluetoothMinor::PeripheralKeyboard,
                    3 => BluetoothMinor::PeripheralKeyboardAndPointer,
                    _ => BluetoothMinor::Uncategorized,
                }
            }
            BluetoothMajor::Imaging => BluetoothMinor::Uncategorized,
            BluetoothMajor::Wearable => match minor {
                BluetoothMinorClass::WearableWristwatch => BluetoothMinor::WearableWristwatch,
                BluetoothMinorClass::WearablePager => BluetoothMinor::WearablePager,
                BluetoothMinorClass::WearableJacket => BluetoothMinor::WearableJacket,
                BluetoothMinorClass::WearableHelmet => BluetoothMinor::WearableHelmet,
                BluetoothMinorClass::WearableGlasses => BluetoothMinor::WearableGlasses,
                _ => BluetoothMinor::Uncategorized,
            },
            BluetoothMajor::Toy => match minor {
                BluetoothMinorClass::ToyRobot => BluetoothMinor::ToyRobot,
                BluetoothMinorClass::ToyVehicle => BluetoothMinor::ToyVehicle,
                BluetoothMinorClass::ToyDoll => BluetoothMinor::ToyDoll,
                BluetoothMinorClass::ToyController => BluetoothMinor::ToyController,
                BluetoothMinorClass::ToyGame => BluetoothMinor::ToyGame,
                _ => BluetoothMinor::Uncategorized,
            },
            BluetoothMajor::Health => match minor {
                BluetoothMinorClass::HealthBloodPressureMonitor => {
                    BluetoothMinor::HealthBloodPressureMonitor
                }
                BluetoothMinorClass::HealthThermometer => BluetoothMinor::HealthThermometer,
                BluetoothMinorClass::HealthWeighingScale => BluetoothMinor::HealthWeighingScale,
                BluetoothMinorClass::HealthGlucoseMeter => BluetoothMinor::HealthGlucoseMeter,
                BluetoothMinorClass::HealthPulseOximeter => BluetoothMinor::HealthPulseOximeter,
                BluetoothMinorClass::HealthHeartRateMonitor => {
                    BluetoothMinor::HealthHeartRateMonitor
                }
                BluetoothMinorClass::HealthHealthDataDisplay => {
                    BluetoothMinor::HealthHealthDataDisplay
                }
                BluetoothMinorClass::HealthStepCounter => BluetoothMinor::HealthStepCounter,
                BluetoothMinorClass::HealthBodyCompositionAnalyzer => {
                    BluetoothMinor::HealthBodyCompositionAnalyzer
                }
                BluetoothMinorClass::HealthPeakFlowMonitor => BluetoothMinor::HealthPeakFlowMonitor,
                BluetoothMinorClass::HealthMedicationMonitor => {
                    BluetoothMinor::HealthMedicationMonitor
                }
                BluetoothMinorClass::HealthKneeProsthesis => BluetoothMinor::HealthKneeProsthesis,
                BluetoothMinorClass::HealthAnkleProsthesis => BluetoothMinor::HealthAnkleProsthesis,
                BluetoothMinorClass::HealthGenericHealthManager => {
                    BluetoothMinor::HealthGenericHealthManager
                }
                BluetoothMinorClass::HealthPersonalMobilityDevice => {
                    BluetoothMinor::HealthPersonalMobilityDevice
                }
                _ => BluetoothMinor::Uncategorized,
            },
            BluetoothMajor::Unkown => BluetoothMinor::Uncategorized,
        }
    }
}

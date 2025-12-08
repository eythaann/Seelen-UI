// all this structs are based on official docs https://www.bluetooth.com/specifications/assigned-numbers
#[cfg(test)]
mod build_low_energy_enums;

pub mod enums;
pub mod low_energy_enums;

use enums::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct BluetoothDevice {
    pub id: String,
    pub name: String,
    pub address: u64,
    pub major_service_classes: Vec<BluetoothMajorServiceClass>,
    pub major_class: BluetoothMajorClass,
    pub minor_class: BluetoothMinorClass,
    /// only available for low energy devices
    pub appearance: Option<low_energy_enums::BLEAppearance>,
    pub connected: bool,
    pub paired: bool,
    pub can_pair: bool,
    pub can_disconnect: bool,
    pub is_low_energy: bool,
}

impl BluetoothDevice {
    fn map_services_classes(class: u32) -> Vec<BluetoothMajorServiceClass> {
        use BluetoothMajorServiceClass::*;
        [
            LimitedDiscoverableMode,
            LowEnergyAudio,
            Reserved,
            Positioning,
            Networking,
            Rendering,
            Capturing,
            ObjectTransfer,
            Audio,
            Telephony,
            Information,
        ]
        .into_iter()
        .filter(|&service| class & service as u32 != 0)
        .collect()
    }

    pub fn get_parts_of_class(
        class: u32,
    ) -> (
        Vec<BluetoothMajorServiceClass>,
        BluetoothMajorClass,
        BluetoothMinorClass,
    ) {
        let major_service_classes = class >> 13;
        let major_service_classes = Self::map_services_classes(major_service_classes);

        let major_class = (class >> 8) & 0b11111; // 5 bits
        let major_class = BluetoothMajorClass::from(major_class as u8);

        let minor_class = ((class >> 2) & 0b111111) as u8; // 6 bits
        let minor_class = match major_class {
            BluetoothMajorClass::Miscellaneous => BluetoothMinorClass::Miscellaneous {
                unused: minor_class,
            },
            BluetoothMajorClass::Computer => {
                BluetoothMinorClass::Computer(BluetoothComputerMinor::from(minor_class))
            }
            BluetoothMajorClass::Phone => {
                BluetoothMinorClass::Phone(BluetoothPhoneMinor::from(minor_class))
            }
            BluetoothMajorClass::NetworkAccessPoint => {
                let minor_class = minor_class >> 3 & 0b111; // 3 bits
                let sub_minor_class = minor_class & 0b111; // 3 bits
                BluetoothMinorClass::NetworkAccessPoint(
                    BluetoothNetworkMinor::from(minor_class),
                    BluetoothNetworkSubMinor::from(sub_minor_class),
                )
            }
            BluetoothMajorClass::AudioVideo => {
                BluetoothMinorClass::AudioVideo(BluetoothAudioVideoMinor::from(minor_class))
            }
            BluetoothMajorClass::Peripheral => {
                let minor_class = minor_class >> 4 & 0b11; // 2 bits
                let sub_minor_class = minor_class & 0b1111; // 4 bits
                BluetoothMinorClass::Peripheral(
                    BluetoothPeripheralMinor::from(minor_class),
                    BluetoothPeripheralSubMinor::from(sub_minor_class),
                )
            }
            BluetoothMajorClass::Imaging => {
                let minor_class = minor_class >> 2 & 0b1111; // 4 bits
                let sub_minor_class = minor_class & 0b11; // 2 bits

                use BluetoothImagingMinor::*;
                let flags: Vec<BluetoothImagingMinor> = [Display, Camera, Scanner, Printer]
                    .into_iter()
                    .filter(|&flag| minor_class & flag as u8 != 0)
                    .collect();

                BluetoothMinorClass::Imaging(flags, BluetoothImagingSubMinor::from(sub_minor_class))
            }
            BluetoothMajorClass::Wearable => {
                BluetoothMinorClass::Wearable(BluetoothWearableMinor::from(minor_class))
            }
            BluetoothMajorClass::Toy => {
                BluetoothMinorClass::Toy(BluetoothToyMinor::from(minor_class))
            }
            BluetoothMajorClass::Health => {
                BluetoothMinorClass::Health(BluetoothHealthMinor::from(minor_class))
            }
            BluetoothMajorClass::Uncategorized => BluetoothMinorClass::Uncategorized {
                unused: minor_class,
            },
        };

        (major_service_classes, major_class, minor_class)
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct BluetoothDevicePairShowPinRequest {
    pub pin: String,
    pub confirmation_needed: bool,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "needs")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub enum DevicePairingNeededAction {
    /// No extra action is needed
    None,
    /// The user only needs to confirm the pairing
    ConfirmOnly,
    /// Should be displayed to the user to be inserted in the other device
    DisplayPin { pin: String },
    /// An input pin should be provided
    ProvidePin,
    /// Pin should be displayed to the user and confirm that is the same as the other device
    ConfirmPinMatch { pin: String },
    /// An input pin should be provided
    ProvidePasswordCredential,
    /// An input address should be provided
    ProvideAddress,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct DevicePairingAnswer {
    pub accept: bool,
    pub pin: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub address: Option<String>,
}

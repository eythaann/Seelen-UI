// all this structs are based on official docs https://www.bluetooth.com/specifications/assigned-numbers
#[cfg(test)]
mod build_enums;

#[rustfmt::skip]
pub mod low_energy_enums;
#[rustfmt::skip]
pub mod class_of_device_enums;

use class_of_device_enums::{BluetoothClass, BluetoothMajorServiceClass};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct BluetoothDevice {
    pub id: String,
    pub name: String,
    pub address: u64,
    pub major_service_classes: Vec<BluetoothMajorServiceClass>,
    pub class: BluetoothClass,
    /// only available for low energy devices
    pub appearance: Option<low_energy_enums::BLEAppearance>,
    pub connected: bool,
    pub paired: bool,
    pub can_pair: bool,
    pub can_disconnect: bool,
    pub can_connect: bool,
    pub is_low_energy: bool,
}

impl BluetoothDevice {
    pub fn get_parts_of_class(class: u32) -> (Vec<BluetoothMajorServiceClass>, BluetoothClass) {
        let major_service_classes = BluetoothMajorServiceClass::from_bits(class >> 13);

        let major = ((class >> 8) & 0b11111) as u8; // 5 bits
        let minor = ((class >> 2) & 0b111111) as u8; // 6 bits
        let class = BluetoothClass::from_major_and_minor(major, minor);

        (major_service_classes, class)
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

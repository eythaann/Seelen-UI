use crossbeam_channel::bounded;
use windows::Devices::Enumeration::{
    DeviceInformationCustomPairing, DevicePairingKinds, DevicePairingRequestedEventArgs,
};
use windows::Foundation::TypedEventHandler;

use crate::{event_manager, log_error, trace_lock};

use crate::error::Result;

use super::BLUETOOTH_MANAGER;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothPairEvent {
    ShowPin(String, bool),
    RequestPin(),
    Confirm(bool, String),
}

#[derive(Debug)]
pub struct BluetoothPairManager {}

unsafe impl Send for BluetoothPairManager {}
unsafe impl Sync for BluetoothPairManager {}
unsafe impl Send for BluetoothPairEvent {}

event_manager!(BluetoothPairManager, BluetoothPairEvent);

impl BluetoothPairManager {
    pub async fn pair(address: u64) -> Result<()> {
        let pairing = {
            let manager = trace_lock!(BLUETOOTH_MANAGER);
            let pair_target = manager.discovered_items.get(&address);
            if let Some(pair_target) = pair_target {
                if let Some(device) = &pair_target.inner {
                    device.DeviceInformation()?.Pairing()?.Custom().ok()
                } else if let Some(device) = &pair_target.inner_le {
                    device.DeviceInformation()?.Pairing()?.Custom().ok()
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(pairing) = pairing {
            let pair_handler =
                pairing.PairingRequested(&TypedEventHandler::new(BluetoothPairManager::on_pair))?;
            let _ = pairing
                .PairAsync(
                    DevicePairingKinds::ConfirmOnly
                        | DevicePairingKinds::ConfirmPinMatch
                        | DevicePairingKinds::DisplayPin
                        | DevicePairingKinds::ProvidePin,
                )?
                .await?;
            pairing.RemovePairingRequested(pair_handler)?;

            return Ok(());
        }

        Err("Pair was not succesfull!".into())
    }

    pub async fn forget(id: String) -> Result<()> {
        let pairing = {
            let manager = trace_lock!(BLUETOOTH_MANAGER);
            let pair_target = manager.known_items.get(&id);
            if let Some(pair_target) = pair_target {
                if let Some(device) = &pair_target.inner {
                    device.DeviceInformation()?.Pairing().ok()
                } else if let Some(device) = &pair_target.inner_le {
                    device.DeviceInformation()?.Pairing().ok()
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(pairing) = pairing {
            let _ = pairing.UnpairAsync()?.await?;
        }

        Err("Unpair was not succesfull!".into())
    }
}

impl BluetoothPairManager {
    // Pair implemented based on:
    // https://github.com/microsoft/Windows-universal-samples/blob/main/Samples/DeviceEnumerationAndPairing/cs/Scenario9_CustomPairDevice.xaml.cs
    pub(super) fn on_pair(
        _sender: &Option<DeviceInformationCustomPairing>,
        args: &Option<DevicePairingRequestedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(pair) = args {
            let pair_kind = pair.PairingKind()?;
            match pair_kind {
                DevicePairingKinds::ConfirmOnly => pair.Accept()?,
                DevicePairingKinds::DisplayPin => {
                    // We just show the PIN on this side. The ceremony is actually completed when the user enters the PIN
                    // on the target device. We automatically accept here since we can't really "cancel" the operation
                    // from this side.
                    pair.Accept()?;
                    // No need for a deferral since we don't need any decision from the user
                    log_error!(Self::event_tx()
                        .send(BluetoothPairEvent::ShowPin(pair.Pin()?.to_string(), false)));
                }
                DevicePairingKinds::ConfirmPinMatch | DevicePairingKinds::ProvidePin => {
                    // We show the PIN here and the user responds with whether the PIN matches what they see
                    // on the target device. Response comes back and we set it on the PinComparePairingRequestedData
                    // then complete the deferral.
                    let collect_pin_deferral = pair.GetDeferral()?;

                    // For the defer implementation we do not have an async awaiter here, so here the sample differs
                    // from the implementation here.
                    let event = match pair_kind {
                        DevicePairingKinds::ProvidePin => BluetoothPairEvent::RequestPin(),
                        DevicePairingKinds::ConfirmPinMatch => {
                            BluetoothPairEvent::ShowPin(pair.Pin()?.to_string(), true)
                        }
                        _ => return Ok(()), //Impossible
                    };
                    log::trace!("Event: {event:?} for kind: {pair_kind:?}");
                    //Send pair request
                    log_error!(Self::event_tx().send(event));

                    //TODO(Eythaan): from here I can not test the process. It should send this event to the UI, but it not arrives!
                    let (rs, rx) = bounded(1);
                    BluetoothPairManager::subscribe(move |event| {
                        if let BluetoothPairEvent::Confirm(accept, pair_passphrase) = event {
                            log_error!(rs.send((accept, pair_passphrase)));
                        }
                    });
                    if let Some((accept, pair_passphrase)) = rx.into_iter().next() {
                        if accept {
                            match pair_kind {
                                DevicePairingKinds::ProvidePin => {
                                    pair.AcceptWithPin(&pair_passphrase.into())?;
                                }
                                DevicePairingKinds::ConfirmPinMatch => {
                                    pair.Accept()?;
                                }
                                _ => {} //Impossible
                            }
                        }
                        collect_pin_deferral.Complete()?;
                    }
                }
                //DevicePairingKinds::ProvideAddress | DevicePairingKinds::ProvidePasswordCredential => { got no idea what to do or mising implementation! }
                _ => {
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}

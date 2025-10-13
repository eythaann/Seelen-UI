import { BluetoothDevices } from "@seelen-ui/lib";
import type { BluetoothDevice, BluetoothDevicePairShowPinRequest } from "@seelen-ui/lib/types";
import { Icon } from "@shared/components/Icon";
import { cx } from "@shared/styles";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { Button, Input, Tooltip } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { getIconForBTDevice, getMinorAsString } from "../application.ts";

export function BluetoothSelectorEntry(props: {
  device: BluetoothDevice;
  selected: boolean;
  onClick?: () => void;
}) {
  let { device, selected, onClick } = props;

  let [loading, setLoading] = useState(false);
  let [showFields, setShowFields] = useState(false);
  let [passphrase, setPassphrase] = useState("");
  let [showErrors, setShowErrors] = useState(false);
  let [confirmationPhase, setConfirmationPhase] = useState(false);
  let [unsubscribtion, setUnsubscribtion] = useState<UnlistenFn | null>(null);

  const { t } = useTranslation();

  useEffect(() => {
    setShowFields(selected && confirmationPhase);
    setShowErrors(false);
    setPassphrase("");
    setLoading(false);
    if (unsubscribtion !== null) {
      unsubscribtion();
      setUnsubscribtion(null);
    }
    setConfirmationPhase(false);
  }, [selected]);

  const onAction = async (accept: boolean) => {
    setLoading(true);
    if (confirmationPhase) {
      try {
        await BluetoothDevices.confirmPair(accept, passphrase);
      } catch {
        setShowErrors(true);
      } finally {
        setLoading(false);
        setConfirmationPhase(false);
      }
      return;
    }

    if (device.paired) {
      try {
        await BluetoothDevices.forgetDevice(device.id);
      } catch {
        setShowErrors(true);
      } finally {
        setLoading(false);
      }
      return;
    }

    try {
      setUnsubscribtion(await BluetoothDevices.onPairRequest(onPairRequest));
      //TODO(Eythaan): from here I can not test the process. It should send this event to the UI, but it not arrives!
      await BluetoothDevices.pairDevice(device.address);
    } catch {
      setShowErrors(true);
    } finally {
      setLoading(false);
    }
  };

  // idk what that hell void was added as type on a argument, remove that after change it on the library.
  const onPairRequest = (param: BluetoothDevicePairShowPinRequest | null) => {
    if (unsubscribtion) {
      unsubscribtion();
    }
    setUnsubscribtion(null);

    if (param && param.pin !== undefined) {
      setPassphrase(param.pin);
      setShowFields(true);
      setConfirmationPhase(param.confirmationNeeded);
    }

    setLoading(false);
  };

  // TODO(Eythaan): Also the confirmation process did not caried to the end anytime - just teoretically works.
  // Please note that there are 3 run through possible!
  // !!! DevicePairingKinds::ConfirmPinMatch (both side see same and click ok)
  // | DevicePairingKinds::ProvidePin (one side gives pin for auth other side confirm, so the item will disappear after pair success)
  // | DevicePairingKinds::DisplayPin (other side confirm) !!!
  return (
    <div
      key={device.id}
      className={cx("bluetooth-entry", {
        "bluetooth-entry-selected": selected,
      })}
      onClick={onClick}
    >
      <div className="bluetooth-entry-info">
        <Icon
          iconName={device.connected ? "TbBluetoothConnected" : "TbBluetooth"}
          size={20}
        />
        <span className="bluetooth-entry-info-label">{device.name}</span>
        <Tooltip
          title={device.appearance
            ? `${device.appearance.category} - ${device.appearance.subcategory}`
            : `${device.majorClass} - ${getMinorAsString(device.minorClass)}`}
        >
          <Icon iconName={getIconForBTDevice(device)} size={20} />
        </Tooltip>
        {device.isLowEnergy && (
          <Tooltip title={t("bluetooth.lowenergy")}>
            <Icon iconName="MdOutlineEnergySavingsLeaf" size={20} />
          </Tooltip>
        )}
      </div>

      {showFields && (
        <form className="bluetooth-entry-fields">
          <Input
            type="text"
            placeholder={t("bluetooth.placeholder.passphrase")}
            value={passphrase}
            status={showErrors ? "error" : undefined}
            onChange={(e) => setPassphrase(e.currentTarget.value)}
            onPressEnter={() => onAction(true)}
            autoFocus={showFields}
          />
        </form>
      )}

      {selected && (
        <div className="bluetooth-entry-actions">
          <Button
            type={device.paired ? "default" : "primary"}
            onClick={() => onAction(true)}
            loading={loading}
            disabled={loading}
          >
            {device.paired ? t("bluetooth.forget") : t("bluetooth.pair")}
          </Button>
          {confirmationPhase && (
            <Button
              type="default"
              onClick={() => onAction(false)}
              loading={loading}
              disabled={loading}
            >
              {t("bluetooth.cancel")}
            </Button>
          )}
        </div>
      )}
    </div>
  );
}

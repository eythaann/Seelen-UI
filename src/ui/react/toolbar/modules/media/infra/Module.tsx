import { invoke, SeelenCommand } from "@seelen-ui/lib";
import type { MediaToolbarItem } from "@seelen-ui/lib/types";
import { useState } from "react";
import { useSelector } from "react-redux";

import { Item } from "../../item/infra/infra.tsx";

import { Selectors } from "../../shared/store/app.ts";

import { WithMediaControls } from "./MediaControls.tsx";

interface Props {
  module: MediaToolbarItem;
  active?: boolean;
}

function MediaModuleItem({ module, active, ...rest }: Props) {
  const {
    id,
    volume = 0,
    muted: isMuted = true,
  } = useSelector((state: any) => Selectors.mediaOutputs(state).find((d) => d.isDefaultMultimedia)) || {};

  const { volume: inputVolume = 0, muted: inputIsMuted = true } =
    useSelector((state: any) => Selectors.mediaInputs(state).find((d) => d.isDefaultMultimedia)) ||
    {};

  const mediaSession = useSelector((state: any) => Selectors.mediaSessions(state).find((d) => d.default)) || null;

  function onWheel(e: WheelEvent) {
    const isUp = e.deltaY < 0;
    const level = Math.max(0, Math.min(1, volume + (isUp ? 0.02 : -0.02)));
    if (id) {
      invoke(SeelenCommand.SetVolumeLevel, {
        deviceId: id,
        level,
        sessionId: null,
      });
    }
  }

  return (
    <Item
      {...rest}
      onWheel={module.withMediaControls ? onWheel : undefined}
      extraVars={{ volume, isMuted, inputVolume, inputIsMuted, mediaSession }}
      module={module}
      active={active}
    />
  );
}

export function MediaModule({ module }: Props) {
  const [open, setOpen] = useState(false);

  return module.withMediaControls
    ? (
      <WithMediaControls setActive={setOpen}>
        <MediaModuleItem module={module} active={open} />
      </WithMediaControls>
    )
    : <MediaModuleItem module={module} />;
}

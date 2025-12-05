import { cx } from "@shared/styles";
import type { UnlistenFn } from "@tauri-apps/api/event";
import React, { type ImgHTMLAttributes } from "react";

import { iconPackManager } from "./common.ts";
import cs from "./index.module.css";

interface MissingIconProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, "src"> {}

interface MissingIconState {
  src: string | null;
  mask: string | null;
  isAproximatelySquare: boolean;
}

const darkModeQuery = globalThis.matchMedia("(prefers-color-scheme: dark)");
function getMissingIcon(): MissingIconState {
  const icon = iconPackManager.getMissingIcon();
  if (icon) {
    return {
      src: (darkModeQuery.matches ? icon.dark : icon.light) || icon.base,
      mask: icon.mask,
      isAproximatelySquare: icon.isAproximatelySquare,
    };
  }
  return { src: null, mask: null, isAproximatelySquare: false };
}

export class MissingIcon extends React.Component<MissingIconProps, MissingIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: MissingIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = getMissingIcon();

    darkModeQuery.addEventListener("change", this.updateSrc);
    iconPackManager.onChange(this.updateSrc).then((unlistener) => {
      this.unlistener = unlistener;
    });
  }

  componentWillUnmount(): void {
    this.unlistener?.();
    this.unlistener = null;
    darkModeQuery.removeEventListener("change", this.updateSrc);
  }

  updateSrc(): void {
    this.setState(getMissingIcon());
  }

  render(): React.ReactNode {
    const dataProps = Object.entries(this.props)
      .filter(([k]) => k.startsWith("data-"))
      .reduce((acc, [k, v]) => ({ ...acc, [k]: v }), {});

    return (
      <figure {...this.props} className={cx(cs.outer, this.props.className)}>
        <img {...dataProps} src={this.state.src || ""} />
        {this.state.mask && (
          <div
            {...dataProps}
            className={cs.mask}
            style={{ maskImage: `url('${this.state.mask}')` }}
          />
        )}
      </figure>
    );
  }
}

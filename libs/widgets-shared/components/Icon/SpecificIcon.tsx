import { cx } from "@shared/styles";
import type { UnlistenFn } from "@tauri-apps/api/event";
import React, { type ImgHTMLAttributes } from "react";

import { iconPackManager } from "./common.ts";
import cs from "./index.module.css";

interface SpecificIconProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, "src"> {
  name: string;
}

interface SpecificIconState {
  src: string | null;
  mask: string | null;
  isAproximatelySquare: boolean;
}

const darkModeQuery = globalThis.matchMedia("(prefers-color-scheme: dark)");
function getSpecificIcon(name: string): SpecificIconState {
  const icon = iconPackManager.getCustomIcon(name);
  if (icon) {
    return {
      src: (darkModeQuery.matches ? icon.dark : icon.light) || icon.base,
      mask: icon.mask,
      isAproximatelySquare: icon.isAproximatelySquare,
    };
  }
  return { src: null, mask: null, isAproximatelySquare: false };
}

export class SpecificIcon extends React.Component<SpecificIconProps, SpecificIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: SpecificIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = {
      ...getSpecificIcon(this.props.name),
    };

    darkModeQuery.addEventListener("change", this.updateSrc);
    iconPackManager.onChange(this.updateSrc).then((unlistener) => {
      this.unlistener = unlistener;
    });
  }

  componentDidUpdate(prevProps: Readonly<SpecificIconProps>): void {
    if (this.props.name !== prevProps.name) {
      this.updateSrc();
    }
  }

  componentWillUnmount(): void {
    this.unlistener?.();
    this.unlistener = null;
    darkModeQuery.removeEventListener("change", this.updateSrc);
  }

  updateSrc(): void {
    this.setState(getSpecificIcon(this.props.name));
  }

  render(): React.ReactNode {
    const { name: _name, ...imgProps } = this.props;
    if (!this.state.src) {
      return null;
    }

    return (
      <figure {...imgProps} className={cx(cs.outer, imgProps.className)}>
        <img src={this.state.src} />
        {this.state.mask && (
          <div
            className={cs.mask}
            style={{ maskImage: `url('${this.state.mask}')` }}
          />
        )}
      </figure>
    );
  }
}

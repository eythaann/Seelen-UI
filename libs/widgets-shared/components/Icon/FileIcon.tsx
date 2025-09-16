import { IconPackManager } from "@seelen-ui/lib";
import { SeelenCommandGetIconArgs } from "@seelen-ui/lib/types";
import { cx } from "@shared/styles";
import { UnlistenFn } from "@tauri-apps/api/event";
import React, { ImgHTMLAttributes } from "react";

import { iconPackManager } from "./common";
import { MissingIcon } from "./MissingIcon";
import cs from "./index.module.css";

interface FileIconProps extends SeelenCommandGetIconArgs, Omit<ImgHTMLAttributes<HTMLImageElement>, "src"> {
  /** if true, no missing icon will be rendered in case no icon found */
  noFallback?: boolean;
}

interface FileIconState {
  src: string | null;
  mask: string | null;
  isAproximatelySquare: boolean;
}

const darkModeQuery = globalThis.matchMedia("(prefers-color-scheme: dark)");
function getIcon(args: SeelenCommandGetIconArgs): FileIconState {
  const icon = iconPackManager.getIcon(args);
  if (icon) {
    return {
      src: (darkModeQuery.matches ? icon.dark : icon.light) || icon.base,
      mask: icon.mask,
      isAproximatelySquare: icon.isAproximatelySquare,
    };
  }
  return { src: null, mask: null, isAproximatelySquare: false };
}
export class FileIcon extends React.Component<FileIconProps, FileIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: FileIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = getIcon({ path: this.props.path, umid: this.props.umid });

    darkModeQuery.addEventListener("change", this.updateSrc);
    iconPackManager.onChange(this.updateSrc).then((unlistener) => {
      this.unlistener = unlistener;
      // initial extranction request if no icon found
      if (!this.state.src) {
        this.requestIconExtraction();
      }
    });
  }

  componentWillUnmount(): void {
    this.unlistener?.();
    this.unlistener = null;
    darkModeQuery.removeEventListener("change", this.updateSrc);
  }

  componentDidUpdate(
    prevProps: Readonly<FileIconProps>,
    prevState: Readonly<FileIconState>,
  ): void {
    if (
      this.props.path !== prevProps.path || this.props.umid !== prevProps.umid
    ) {
      this.updateSrc();
    }

    if (prevState.src && !this.state.src) {
      this.requestIconExtraction();
    }
  }

  requestIconExtraction(): void {
    IconPackManager.requestIconExtraction({
      path: this.props.path,
      umid: this.props.umid,
    });
  }

  updateSrc(): void {
    this.setState(getIcon({ path: this.props.path, umid: this.props.umid }));
  }

  render(): React.ReactNode {
    const { path: _path, umid: _umid, noFallback, ...imgProps } = this.props;

    const dataProps = Object.entries(imgProps)
      .filter(([k]) => k.startsWith("data-"))
      .reduce((acc, [k, v]) => ({ ...acc, [k]: v }), {});

    if (this.state.src) {
      return (
        <figure
          {...imgProps}
          className={cx(cs.outer, imgProps.className)}
          data-shape={this.state.isAproximatelySquare ? "square" : "unknown"}
        >
          <img {...dataProps} src={this.state.src} />
          {this.state.mask && (
            <div
              {...dataProps}
              className={cx(cs.mask, "sl-mask")}
              style={{ maskImage: `url('${this.state.mask}')` }}
            />
          )}
        </figure>
      );
    }

    if (noFallback) {
      return null;
    }

    return <MissingIcon {...imgProps} />;
  }
}

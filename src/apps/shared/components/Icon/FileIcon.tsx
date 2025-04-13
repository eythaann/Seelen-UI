import { GetIconArgs, IconPackManager } from '@seelen-ui/lib';
import { cx } from '@shared/styles';
import { UnlistenFn } from '@tauri-apps/api/event';
import React, { ImgHTMLAttributes } from 'react';

import { iconPackManager } from './common';
import { MissingIcon } from './MissingIcon';
import cs from './index.module.css';

interface FileIconProps extends GetIconArgs, Omit<ImgHTMLAttributes<HTMLImageElement>, 'src'> {
  /** if true, no missing icon will be rendered in case no icon found */
  noFallback?: boolean;
}

interface FileIconState {
  src: string | null;
  mask?: string | null;
}

const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
function getIcon(args: GetIconArgs): {
  src: string | null;
  mask?: string | null;
} {
  const icon = iconPackManager.getIcon(args);
  if (icon && typeof icon === 'object') {
    return {
      src: darkModeQuery.matches ? icon.dark : icon.light,
      mask: icon.mask,
    };
  }
  return { src: icon };
}
export class FileIcon extends React.Component<FileIconProps, FileIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: FileIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = getIcon({ path: this.props.path, umid: this.props.umid });

    darkModeQuery.addEventListener('change', this.updateSrc);
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
  }

  componentDidUpdate(prevProps: Readonly<FileIconProps>, prevState: Readonly<FileIconState>): void {
    if (this.props.path !== prevProps.path || this.props.umid !== prevProps.umid) {
      this.updateSrc();
    }

    if (prevState.src && !this.state.src) {
      this.requestIconExtraction();
    }
  }

  requestIconExtraction(): void {
    IconPackManager.extractIcon({
      path: this.props.path,
      umid: this.props.umid,
    });
  }

  updateSrc(): void {
    this.setState({
      ...getIcon({ path: this.props.path, umid: this.props.umid }),
    });
  }

  render(): React.ReactNode {
    const { path: _path, umid: _umid, noFallback, ...imgProps } = this.props;

    if (this.state.src) {
      return (
        <figure {...imgProps} className={cx(cs.outer, imgProps.className)}>
          <img src={this.state.src} />
          {this.state.mask && (
            <div
              className={cx(cs.mask, 'sl-mask')}
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

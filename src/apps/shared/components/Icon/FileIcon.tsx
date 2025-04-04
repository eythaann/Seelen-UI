import { GetIconArgs, IconPackManager } from '@seelen-ui/lib';
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
}

const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
function getIcon(args: GetIconArgs): string | null {
  const icon = iconPackManager.getIcon(args);
  if (icon && typeof icon === 'object') {
    return darkModeQuery.matches ? icon.dark : icon.light;
  }
  return icon;
}

export class FileIcon extends React.Component<FileIconProps, FileIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: FileIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = {
      src: getIcon({ path: this.props.path, umid: this.props.umid }),
    };

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
      src: getIcon({ path: this.props.path, umid: this.props.umid }),
    });
  }

  render(): React.ReactNode {
    const { path: _path, umid: _umid, noFallback, ...imgProps } = this.props;

    if (this.state.src) {
      const style = {
        ...(imgProps.style || {}),
        '--icon-url': `url('${this.state.src}')`,
      } as React.CSSProperties;

      return (
        <figure {...imgProps} style={style}>
          <img src={this.state.src} className={cs.inner} />
        </figure>
      );
    }

    if (noFallback) {
      return null;
    }

    return <MissingIcon {...imgProps} />;
  }
}

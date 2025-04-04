import { UnlistenFn } from '@tauri-apps/api/event';
import React, { ImgHTMLAttributes } from 'react';

import { iconPackManager } from './common';
import cs from './index.module.css';

interface MissingIconProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, 'src'> {}

interface MissingIconState {
  src: string | null;
}

const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
function getMissingIconSrc(): string | null {
  const icon = iconPackManager.getMissingIcon();
  if (icon && typeof icon === 'object') {
    return darkModeQuery.matches ? icon.dark : icon.light;
  }
  return icon;
}

export class MissingIcon extends React.Component<MissingIconProps, MissingIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: MissingIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = {
      src: getMissingIconSrc(),
    };

    darkModeQuery.addEventListener('change', this.updateSrc);
    iconPackManager.onChange(this.updateSrc).then((unlistener) => {
      this.unlistener = unlistener;
    });
  }

  componentWillUnmount(): void {
    this.unlistener?.();
    this.unlistener = null;
    darkModeQuery.removeEventListener('change', this.updateSrc);
  }

  updateSrc(): void {
    this.setState({
      src: getMissingIconSrc(),
    });
  }

  render(): React.ReactNode {
    const style = {
      ...(this.props.style || {}),
      '--icon-url': `url('${this.state.src}')`,
    } as React.CSSProperties;

    return (
      <figure {...this.props} style={style}>
        <img src={this.state.src || ''} className={cs.inner} />
      </figure>
    );
  }
}

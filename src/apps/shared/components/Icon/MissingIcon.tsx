import { UnlistenFn } from '@tauri-apps/api/event';
import React, { ImgHTMLAttributes } from 'react';

import { iconPackManager } from './common';

interface MissingIconProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, 'src'> {}

interface MissingIconState {
  src: string | null;
}

export class MissingIcon extends React.Component<MissingIconProps, MissingIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: MissingIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = {
      src: iconPackManager.getMissingIcon(),
    };

    iconPackManager.onChange(this.updateSrc).then((unlistener) => {
      this.unlistener = unlistener;
    });
  }

  componentWillUnmount(): void {
    this.unlistener?.();
    this.unlistener = null;
  }

  updateSrc(): void {
    this.setState({
      src: iconPackManager.getMissingIcon(),
    });
  }

  render(): React.ReactNode {
    const style = {
      ...(this.props.style || {}),
      '--icon-url': `url('${this.state.src}')`,
    } as React.CSSProperties;

    return (
      <figure {...this.props} style={style}>
        <img src={this.state.src || ''} style={{ height: '100%' }} />
      </figure>
    );
  }
}

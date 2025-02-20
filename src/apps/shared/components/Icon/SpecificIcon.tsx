import { UnlistenFn } from '@tauri-apps/api/event';
import React, { ImgHTMLAttributes } from 'react';

import { iconPackManager } from './common';
import cs from './index.module.css';

interface SpecificIconProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, 'src'> {
  name: string;
}

interface SpecificIconState {
  src: string | null;
}

export class SpecificIcon extends React.Component<SpecificIconProps, SpecificIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: SpecificIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = {
      src: iconPackManager.getSpecificIcon(this.props.name),
    };

    iconPackManager.onChange(this.updateSrc).then((unlistener) => {
      this.unlistener = unlistener;
    });
  }

  componentDidUpdate(prevProps: Readonly<SpecificIconProps>): void {
    if (this.props.name !== prevProps.name) {
      this.updateSrc();
    }
  }

  updateSrc(): void {
    this.setState({
      src: iconPackManager.getSpecificIcon(this.props.name),
    });
  }

  render(): React.ReactNode {
    const { name: _name, ...imgProps } = this.props;
    if (!this.state.src) {
      return null;
    }

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
}

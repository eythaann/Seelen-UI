import { cx } from '@shared/styles';
import { UnlistenFn } from '@tauri-apps/api/event';
import React, { ImgHTMLAttributes } from 'react';

import { iconPackManager } from './common';
import cs from './index.module.css';

interface SpecificIconProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, 'src'> {
  name: string;
}

interface SpecificIconState {
  src: string | null;
  mask?: string | null;
}

const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
function getSpecificIcon(name: string): {
  src: string | null;
  mask?: string | null;
} {
  const icon = iconPackManager.getSpecificIcon(name);
  if (icon && typeof icon === 'object') {
    return {
      src: darkModeQuery.matches ? icon.dark : icon.light,
      mask: icon.mask,
    };
  }
  return { src: icon };
}

export class SpecificIcon extends React.Component<SpecificIconProps, SpecificIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: SpecificIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = {
      ...getSpecificIcon(this.props.name),
    };

    darkModeQuery.addEventListener('change', this.updateSrc);
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
      ...getSpecificIcon(this.props.name),
    });
  }

  render(): React.ReactNode {
    const { name: _name, ...imgProps } = this.props;
    if (!this.state.src) {
      return null;
    }

    return (
      <figure {...imgProps} className={cx(cs.outer, imgProps.className)}>
        <img src={this.state.src} className={cs.inner} />
        {this.state.mask && (
          <div
            className={cx(cs.mask, 'sl-mask')}
            style={{ maskImage: `url('${this.state.mask}')` }}
          />
        )}
      </figure>
    );
  }
}

import { convertFileSrc } from '@tauri-apps/api/core';
import React from 'react';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { useIcon } from 'src/apps/shared/hooks';

import { Icon } from '../../../../shared/components/Icon';

interface StringToElementProps {
  text: string;
}

interface ExeIconProps {
  path?: string | null;
  umid?: string | null;
  size: number;
}

function ExeIcon({ path, umid, size }: ExeIconProps) {
  const icon = useIcon({ path, umid }) || convertFileSrc(LAZY_CONSTANTS.MISSING_ICON_PATH);

  return <img src={icon} style={{ width: size }} />;
}

export class StringToElement extends React.PureComponent<StringToElementProps> {
  static imgPrefix = 'IMG:';
  static iconPrefix = 'ICON:';
  static exePrefix = 'EXE:';

  static getIcon(name: string, size = 16) {
    if (!name) {
      return '';
    }
    return `[ICON:${JSON.stringify({ name, size })}]`;
  }

  static imgFromUrl(url: string, size = 16) {
    if (!url) {
      return '';
    }
    return `[IMG:${JSON.stringify({ url, size })}]`;
  }

  static imgFromPath(path?: string | null, size = 16) {
    if (!path) {
      return '';
    }
    return StringToElement.imgFromUrl(convertFileSrc(path), size);
  }

  static imgFromExe(path?: string | null, umid?: string | null, size = 16) {
    if (!path) {
      return '';
    }
    return `[EXE:${JSON.stringify({ path, umid, size })}]`;
  }

  constructor(props: StringToElementProps) {
    super(props);
  }

  isImg() {
    return this.props.text.startsWith(StringToElement.imgPrefix);
  }

  isIcon() {
    return this.props.text.startsWith(StringToElement.iconPrefix);
  }

  isExe() {
    return this.props.text.startsWith(StringToElement.exePrefix);
  }

  render() {
    if (this.isExe()) {
      const json = this.props.text.slice(StringToElement.exePrefix.length);
      const { path, umid, size } = JSON.parse(json);

      return <ExeIcon path={path} umid={umid} size={size} />;
    }

    if (this.isImg()) {
      const json = this.props.text.slice(StringToElement.imgPrefix.length);
      const { url, size } = JSON.parse(json);
      return <img src={url} style={{ width: size }} />;
    }

    if (this.isIcon()) {
      const json = this.props.text.slice(StringToElement.iconPrefix.length);
      const { name, size } = JSON.parse(json);
      return <Icon iconName={name} size={size} />;
    }

    return <span>{this.props.text}</span>;
  }
}

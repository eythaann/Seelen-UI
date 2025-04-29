import { convertFileSrc } from '@tauri-apps/api/core';
import React from 'react';

import { EvaluateAction } from '../app';

import { FileIcon, Icon } from '../../../../shared/components/Icon';

interface StringToElementProps {
  text: string;
}

export class StringToElement extends React.PureComponent<StringToElementProps> {
  static buttonPrefix = 'BUTTON:';
  static iconPrefix = 'ICON:';
  static imgPrefix = 'IMG:';
  static exePrefix = 'EXE:';

  static getIcon(name: string, size = 16) {
    if (!name) {
      return '';
    }
    return `${StringToElement.iconPrefix}${JSON.stringify({ name, size })}`;
  }

  static getButton(opt: { onClick: string; content: string }) {
    return `${StringToElement.buttonPrefix}${JSON.stringify(opt)}`;
  }

  static imgFromUrl(url: string, size = 16) {
    if (!url) {
      return '';
    }
    return `${StringToElement.imgPrefix}${JSON.stringify({ url, size })}`;
  }

  static imgFromPath(path?: string | null, size = 16) {
    if (!path) {
      return '';
    }
    return StringToElement.imgFromUrl(convertFileSrc(path), size);
  }

  static imgFromApp(path?: string | null, umid?: string | null, size = 16) {
    if (!path && !umid) {
      return '';
    }
    return `${StringToElement.exePrefix}${JSON.stringify({ path, umid, size })}`;
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

  isButton() {
    return this.props.text.startsWith(StringToElement.buttonPrefix);
  }

  isExe() {
    return this.props.text.startsWith(StringToElement.exePrefix);
  }

  render() {
    if (this.isExe()) {
      const json = this.props.text.slice(StringToElement.exePrefix.length);
      const { path, umid, size } = JSON.parse(json);

      return <FileIcon path={path} umid={umid} style={{ width: size }} />;
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

    if (this.isButton()) {
      const json = this.props.text.slice(StringToElement.buttonPrefix.length);
      const { onClick, content } = JSON.parse(json);

      return <button onClick={() => EvaluateAction(onClick, {})}>
        <StringToElement text={content} />
      </button>;
    }

    return <span>{this.props.text}</span>;
  }
}

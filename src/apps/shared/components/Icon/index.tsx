import { IconBaseProps } from 'react-icons';
import * as ai from 'react-icons/ai';
import * as bi from 'react-icons/bi';
import * as bs from 'react-icons/bs';
import * as cg from 'react-icons/cg';
import * as ci from 'react-icons/ci';
import * as di from 'react-icons/di';
import * as fa from 'react-icons/fa';
import * as fa6 from 'react-icons/fa6';
import * as fc from 'react-icons/fc';
import * as fi from 'react-icons/fi';
import * as gi from 'react-icons/gi';
import * as go from 'react-icons/go';
import * as gr from 'react-icons/gr';
import * as hi from 'react-icons/hi';
import * as hi2 from 'react-icons/hi2';
import * as im from 'react-icons/im';
import * as io from 'react-icons/io';
import * as io5 from 'react-icons/io5';
import * as lia from 'react-icons/lia';
import * as lu from 'react-icons/lu';
import * as md from 'react-icons/md';
import * as pi from 'react-icons/pi';
import * as ri from 'react-icons/ri';
import * as rx from 'react-icons/rx';
import * as si from 'react-icons/si';
import * as sl from 'react-icons/sl';
import * as tb from 'react-icons/tb';
import * as tfi from 'react-icons/tfi';
import * as ti from 'react-icons/ti';
import * as vsc from 'react-icons/vsc';
import * as wi from 'react-icons/wi';

import cs from './index.module.css';

export type Icon = keyof typeof icons;
const icons = {
  ...ai,
  ...bi,
  ...bs,
  ...cg,
  ...ci,
  ...di,
  ...fa,
  ...fa6,
  ...fc,
  ...fi,
  ...gi,
  ...go,
  ...gr,
  ...hi,
  ...hi2,
  ...im,
  ...io,
  ...io5,
  ...lia,
  ...lu,
  ...md,
  ...pi,
  ...ri,
  ...rx,
  ...si,
  ...sl,
  ...tb,
  ...tfi,
  ...ti,
  ...vsc,
  ...wi,
};

export const exposedIconsRegex = /\[ICON:(.*?)\]/g;
export const exposedIcons = Object.keys(icons).reduce((acc, icon) => {
  acc[icon] = `[ICON:${icon}]`;
  return acc;
}, {} as any);

export function isValidIconName(str: string) {
  const [name] = str.split(':');
  return !!icons[name as Icon];
}

interface typesPropsIcon {
  iconName: Icon;
  propsIcon?: IconBaseProps;
}

export function Icon(props: typesPropsIcon) {
  const { iconName, propsIcon } = props;

  const Icon = icons[iconName] || null;

  if (!Icon) {
    return null;
  }

  return (
    <i className={cs.icon}>
      <Icon {...propsIcon} />
    </i>
  );
}

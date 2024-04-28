import { IconBaseProps, IconType } from 'react-icons';
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

const libs = {
  ai: ai,
  bi: bi,
  bs: bs,
  cg: cg,
  ci: ci,
  di: di,
  fa: fa,
  fa6: fa6,
  fc: fc,
  fi: fi,
  gi: gi,
  go: go,
  gr: gr,
  hi: hi,
  hi2: hi2,
  im: im,
  io: io,
  io5: io5,
  lia: lia,
  lu: lu,
  md: md,
  pi: pi,
  ri: ri,
  rx: rx,
  si: si,
  sl: sl,
  tb: tb,
  tfi: tfi,
  ti: ti,
  vsc: vsc,
  wi: wi,
};

type libs = typeof libs;

interface typesPropsIcon<T extends keyof libs> {
  lib: T;
  iconName: keyof libs[T];
  propsIcon?: IconBaseProps;
}

export function Icon<T extends keyof libs>(props: typesPropsIcon<T>) {
  const { lib, iconName, propsIcon } = props;

  const Icon: IconType | null = libs[lib]?.[iconName] as IconType || null;

  if (!Icon) {
    return null;
  }

  return (
    <i className={cs.icon}>
      <Icon {...propsIcon} />
    </i>
  );
}

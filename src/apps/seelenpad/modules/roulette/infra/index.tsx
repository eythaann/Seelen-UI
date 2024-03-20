import { Tooltip } from 'antd';
import { CSSProperties } from 'react';

import { useAppSelector } from '../../shared/hooks/infra';

import { Selectors } from '../../shared/store/app';
import { SelectDisplayingRoulette } from '../app/slice';

import cs from './index.module.css';

interface BkCustomization {
  layers: CSSProperties[];
}

const DefaultBkStyle: BkCustomization = {
  layers: [
    {
      borderRadius: '50%',
      border: '100px solid rgba(0, 0, 0, 0.8)',
    },
    {
      borderRadius: '50%',
      background: 'radial-gradient(circle, transparent 50%, rgba(0, 0, 0, 0.8) 100%)',
    },
    {
      borderRadius: '50%',
      background: 'radial-gradient(circle, transparent 23%, rgba(0, 0, 0, 0.6) 24%, transparent 45%)',
    },
  ],
};

const Rad = Math.PI / 2;

export function Roulette() {
  const stack = useAppSelector(Selectors.roulette.stack);
  const rotationStep = useAppSelector(Selectors.roulette.rotationStep);
  const roulette = useAppSelector(SelectDisplayingRoulette);

  const selectedItemIdx = roulette.items.at(rotationStep % roulette.items.length)!.position;

  let arcBetweenItems = (4 * Rad) / roulette.items.length;

  return (
    <div className={cs.roulette}>
      <div className={cs.drag} data-tauri-drag-region>
        ➕
      </div>

      <div className={cs.backgrounds}>
        {DefaultBkStyle.layers.map((style, idx) => (
          <div key={idx} style={style} />
        ))}
      </div>

      <div
        key={stack.length}
        className={cs.items}
        style={{ transform: `rotate(${arcBetweenItems * rotationStep * -1}rad)` }}
      >
        {roulette.items.map((item, idx) => {
          const deg = arcBetweenItems * idx;
          const radio = 100;
          const top = Math.cos(deg) * radio * -1;
          const left = Math.sin(deg) * radio;

          const style: CSSProperties = {
            top,
            left,
            transform: `translate(-50%, -50%) rotate(${arcBetweenItems * rotationStep}rad)`,
          };

          if (item.position === selectedItemIdx) {
            style.transform += ' scale(3)';
          }

          return (
            <div key={item.position} className={cs.item} style={style}>
              <Tooltip
                title={item.label}
                placement="bottom"
                arrow={false}
                open={item.position === selectedItemIdx}
                rootClassName={cs.tooltip}
              >
                {item.icon}
              </Tooltip>
            </div>
          );
        })}
      </div>
    </div>
  );
}

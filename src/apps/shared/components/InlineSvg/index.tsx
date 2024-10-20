import { HTMLAttributes, useEffect, useState } from 'react';

import { cx } from '../../../../apps/shared/styles';

import cs from './index.module.css';

interface Props extends HTMLAttributes<HTMLElement> {
  src: string;
}

const InlineSVG = ({ src, className, ...rest }: Props) => {
  const [svgContent, setSvgContent] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchSVG = async () => {
      try {
        const response = await fetch(src);
        if (!response.ok) {
          throw new Error(`Failed to fetch SVG: ${response.statusText}`);
        }
        const svgText = await response.text();
        setSvgContent(svgText);
      } catch (err: any) {
        setError(err?.message);
      }
    };

    fetchSVG();
  }, [src]);

  if (error || !svgContent) {
    return null;
  }

  return (
    <i
      {...rest}
      className={cx(cs.inlineSvg, className)}
      dangerouslySetInnerHTML={{ __html: svgContent }}
    />
  );
};

export default InlineSVG;

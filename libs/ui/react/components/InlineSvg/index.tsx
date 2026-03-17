import { cx } from "libs/ui/react/utils/styling";
import { forwardRef, type HTMLAttributes, useEffect, useState } from "react";

import cs from "./index.module.css";

interface Props extends HTMLAttributes<HTMLElement> {
  src: string;
}

const InlineSVG = forwardRef<HTMLElement, Props>(({ src, className, ...rest }, ref) => {
  const [svgContent, setSvgContent] = useState<string | null>(null);

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
        console.error(err);
      }
    };

    fetchSVG();
  }, [src]);

  return (
    <i
      ref={ref}
      {...rest}
      className={cx(cs.inlineSvg, className)}
      dangerouslySetInnerHTML={{ __html: svgContent ?? "" }}
    />
  );
});

export default InlineSVG;

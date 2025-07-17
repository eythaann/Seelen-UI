import { cx } from '../../../shared/styles';
import cs from './index.module.css';

interface Props extends React.HTMLAttributes<HTMLDivElement> {
  width?: number;
  height?: number;
}

export function Monitor({ children, className, width = 1920, height = 1080, ...props }: Props) {
  // const seleenWallpaper = useSelector(newSelectors.wall.backgrounds);

  // const controls = useAnimationControls();

  // const wallpaper = seleenWallpaper[0]?.path;

  const style: React.CSSProperties = {
    aspectRatio: `${width} / ${height}`,
  };
  if (width > height) {
    style.width = '100%';
  } else {
    style.height = '100%';
  }

  return (
    <div className={cs.monitorContainer} {...props}>
      <div
        className={cx(cs.monitor, className)}
        style={style}
      >
        <div className={cs.screen}>
          {/* {wallpaper && (
            <motion.img
              className={cs.wallpaper}
              src={convertFileSrc(wallpaper)}
              initial={{ opacity: 0 }}
              animate={controls}
              onLoad={() => {
                controls.start({ opacity: 1, transition: { duration: 0.3, ease: 'linear' } });
              }}
            />
          )} */}
          {children}
        </div>
      </div>
    </div>
  );
}

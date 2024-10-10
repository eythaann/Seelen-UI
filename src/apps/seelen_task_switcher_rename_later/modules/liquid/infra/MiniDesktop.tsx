import { Desktop } from '../../shared/store/domain';

export function MiniDesktop(props: { desk: Desktop }) {
  return <div className="mini-desktop">{props.desk.name}</div>;
}

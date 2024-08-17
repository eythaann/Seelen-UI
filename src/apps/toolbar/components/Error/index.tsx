import cs from './index.module.css';

interface Props {
  msg?: string;
}

export function ErrorFallback({ msg = 'Something went wrong' }: Props) {
  return <div className={cs.error}>{msg}</div>;
}

import { useEffect, useState } from 'react';

import cs from './infra.module.css';

import { Layout, RELATION_ASPECT_RATIO } from './domain';

interface Props {
  containerPadding: number;
  workspacePadding: number;
}

const BSPLayoutExample = ({ containerPadding, workspacePadding }: Props) => {
  const [counter, setCounter] = useState(1);

  useEffect(() => {
    setInterval(() => setCounter((v) => (v >= 4 ? 1 : v + 1)), 1000);
  }, []);

  const style = {
    gap: containerPadding / RELATION_ASPECT_RATIO,
  };

  return (
    <div
      className={cs.colums}
      style={{
        ...style,
        padding: workspacePadding / RELATION_ASPECT_RATIO,
      }}
    >
      <div className={cs.window} />
      {counter >= 2 && (
        <div className={cs.rows} style={style}>
          <div className={cs.window} />
          {counter >= 3 && (
            <div className={cs.colums} style={style}>
              <div className={cs.window} />
              {counter >= 4 && <div className={cs.window} />}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

const ColumsLayoutExample = ({ containerPadding, workspacePadding }: Props) => {
  const [counter, setCounter] = useState(1);

  useEffect(() => {
    setInterval(() => setCounter((v) => (v >= 4 ? 1 : v + 1)), 1000);
  }, []);

  return (
    <div
      className={cs.colums}
      style={{
        gap: containerPadding / RELATION_ASPECT_RATIO,
        padding: workspacePadding / RELATION_ASPECT_RATIO,
      }}
    >
      <div className={cs.window} />
      {counter >= 2 && <div className={cs.window} />}
      {counter >= 3 && <div className={cs.window} />}
      {counter >= 4 && <div className={cs.window} />}
    </div>
  );
};

const RowsLayoutExample = ({ containerPadding, workspacePadding }: Props) => {
  const [counter, setCounter] = useState(1);

  useEffect(() => {
    setInterval(() => setCounter((v) => (v >= 4 ? 1 : v + 1)), 1000);
  }, []);

  return (
    <div
      className={cs.rows}
      style={{
        gap: containerPadding / RELATION_ASPECT_RATIO,
        padding: workspacePadding / RELATION_ASPECT_RATIO,
      }}
    >
      <div className={cs.window} />
      {counter >= 2 && <div className={cs.window} />}
      {counter >= 3 && <div className={cs.window} />}
      {counter >= 4 && <div className={cs.window} />}
    </div>
  );
};

const HorizontalStackLayoutExample = ({ containerPadding, workspacePadding }: Props) => {
  const [counter, setCounter] = useState(1);

  useEffect(() => {
    setInterval(() => setCounter((v) => (v >= 4 ? 1 : v + 1)), 1000);
  }, []);

  const style = {
    gap: containerPadding / RELATION_ASPECT_RATIO,
  };

  return (
    <div
      className={cs.rows}
      style={{
        ...style,
        padding: workspacePadding / RELATION_ASPECT_RATIO,
      }}
    >
      <div className={cs.window} />
      {counter >= 2 && (
        <div className={cs.colums} style={style}>
          <div className={cs.window} />
          {counter >= 3 && <div className={cs.window} />}
          {counter >= 4 && <div className={cs.window} />}
        </div>
      )}
    </div>
  );
};

const VerticalStackLayoutExample = ({ containerPadding, workspacePadding }: Props) => {
  const [counter, setCounter] = useState(1);

  useEffect(() => {
    setInterval(() => setCounter((v) => (v >= 4 ? 1 : v + 1)), 1000);
  }, []);

  const style = {
    gap: containerPadding / RELATION_ASPECT_RATIO,
  };

  return (
    <div
      className={cs.colums}
      style={{
        ...style,
        padding: workspacePadding / RELATION_ASPECT_RATIO,
      }}
    >
      <div className={cs.window} />
      {counter >= 2 && (
        <div className={cs.rows} style={style}>
          <div className={cs.window} />
          {counter >= 3 && <div className={cs.window} />}
          {counter >= 4 && <div className={cs.window} />}
        </div>
      )}
    </div>
  );
};

const UltrawideVerticalStackLayoutExample = ({ containerPadding, workspacePadding }: Props) => {
  const [counter, setCounter] = useState(1);

  useEffect(() => {
    setInterval(() => setCounter((v) => (v >= 4 ? 1 : v + 1)), 1000);
  }, []);

  const style = {
    gap: containerPadding / RELATION_ASPECT_RATIO,
  };

  return (
    <div
      className={cs.colums}
      style={{
        ...style,
        padding: workspacePadding / RELATION_ASPECT_RATIO,
      }}
    >
      <div className={cs.window} />
      {counter >= 2 && <div className={cs.window} style={{ flex: counter >= 3 ? 2 : 1 }} />}
      {counter >= 3 && (
        <div className={cs.rows} style={style}>
          <div className={cs.window} />
          {counter >= 4 && <div className={cs.window} />}
          {counter >= 5 && <div className={cs.window} />}
        </div>
      )}
    </div>
  );
};

const GridLayoutExample = ({ containerPadding, workspacePadding }: Props) => {
  const [counter, setCounter] = useState(1);

  useEffect(() => {
    setInterval(() => setCounter((v) => (v >= 8 ? 1 : v + 1)), 1000);
  }, []);

  const style = {
    gap: containerPadding / RELATION_ASPECT_RATIO,
  };

  return (
    <div
      className={cs.colums}
      style={{
        ...style,
        padding: workspacePadding / RELATION_ASPECT_RATIO,
      }}
    >
      <div className={cs.rows} style={style}>
        <div className={cs.window} />
        {counter >= 4 && counter != 5 && <div className={cs.window} />}
        {counter >= 9 && <div className={cs.window} />}
      </div>
      {counter >= 2 && (
        <div className={cs.rows} style={style}>
          <div className={cs.window} />
          {counter >= 3 && <div className={cs.window} />}
          {counter >= 8 && <div className={cs.window} />}
        </div>
      )}
      {counter >= 5 && (
        <div className={cs.rows} style={style}>
          <div className={cs.window} />
          <div className={cs.window} />
          {counter >= 7 && <div className={cs.window} />}
        </div>
      )}
    </div>
  );
};

export const LayoutExamples: Record<Layout, React.JSXElementConstructor<Props>> = {
  [Layout.BSP]: BSPLayoutExample,
  [Layout.COLUMNS]: ColumsLayoutExample,
  [Layout.ROWS]: RowsLayoutExample,
  [Layout.HORIZONTAL_STACK]: HorizontalStackLayoutExample,
  [Layout.VERTICAL_STACK]: VerticalStackLayoutExample,
  [Layout.ULTRAWIDE_VERTICAL_STACK]: UltrawideVerticalStackLayoutExample,
  [Layout.GRID]: GridLayoutExample,
};

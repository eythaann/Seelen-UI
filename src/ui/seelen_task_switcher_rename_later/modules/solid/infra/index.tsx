import { Reorder } from "framer-motion";
import { useDispatch, useSelector } from "react-redux";

import { Actions, Selectors } from "../../shared/store/app";

import { Desktop } from "../../shared/store/domain";

function SolidNav() {
  const desktops = useSelector(Selectors.desktops);

  const dispatch = useDispatch();

  const onReorder = (apps: Desktop[]) => {
    dispatch(Actions.setDesktops(apps));
  };

  return (
    <nav className="desktops-container">
      <Reorder.Group
        values={desktops}
        onReorder={onReorder}
        className="desktop-list"
        axis="y"
        layoutScroll
      >
        {desktops.map((d) => (
          <Reorder.Item key={d.id} value={d} className="desktop-container">
            <button className="desktop">
              <div className="desktop-header">{d.name}</div>
              <img className="desktop-preview" src={d.preview || ""} />
            </button>
          </Reorder.Item>
        ))}
      </Reorder.Group>
      <button className="add-desktop">+</button>
    </nav>
  );
}

const windows = Array.from({ length: 7 }).map((_, i) => ({
  hwnd: i,
  title: `Window ${i}`,
  icon: "",
  preview: "",
}));

function Window({ w }: { w: (typeof windows)[0] }) {
  return (
    <button className="window">
      <div className="window-header">
        <img className="window-header-icon" src={w.icon} />
        <span className="window-header-label">{w.title}</span>
        <div className="window-header-close">X</div>
      </div>
      <img className="window-preview" src={w.preview} />
    </button>
  );
}

export function Solid() {
  return (
    <div className="solid">
      <SolidNav />
      <ul className="focused-desktop-window-list">
        {windows.map((w) => (
          <li key={w.hwnd} className="window-container">
            <Window w={w} />
          </li>
        ))}
      </ul>
    </div>
  );
}

import cs from "./SharedComponents.module.css";

interface ComponentEntry {
  selector: string;
  preview: React.ReactNode;
}

const BUTTON_ENTRIES: ComponentEntry[] = [
  {
    selector: 'button[data-skin="transparent"]',
    preview: <button data-skin="transparent">Example</button>,
  },
  {
    selector: 'button[data-skin="solid"]',
    preview: <button data-skin="solid">Example</button>,
  },
  {
    selector: 'button[data-skin="default"]',
    preview: <button data-skin="default">Example</button>,
  },
  {
    selector: 'div[data-behavior="group"] > button',
    preview: (
      <div data-behavior="group">
        <button data-skin="default">One</button>
        <button data-skin="default">Two</button>
        <button data-skin="default">Three</button>
      </div>
    ),
  },
];

const SURFACE_ENTRIES: ComponentEntry[] = [
  {
    selector: ".slu-std-popover",
    preview: (
      <div className="slu-std-popover" style={{ position: "relative", margin: 0 }}>
        Popover content
      </div>
    ),
  },
];

const INPUT_ENTRIES: ComponentEntry[] = [
  {
    selector: 'input[type="text"][data-skin="default"]',
    preview: <input type="text" data-skin="default" placeholder="Example" />,
  },
  {
    selector: 'input[type="text"][data-skin="transparent"]',
    preview: <input type="text" data-skin="transparent" placeholder="Example" />,
  },
  {
    selector: 'input[type="range"][data-skin="flat"]',
    preview: <input type="range" data-skin="flat" defaultValue={50} />,
  },
  {
    selector: 'input[type="checkbox"][data-skin="switch"]',
    preview: <input type="checkbox" data-skin="switch" defaultChecked />,
  },
  {
    selector: 'select[data-skin="default"]',
    preview: (
      <select data-skin="default">
        <option>Option A</option>
        <option>Option B</option>
        <option>Option C</option>
      </select>
    ),
  },
  {
    selector: 'select[data-skin="transparent"]',
    preview: (
      <select data-skin="transparent">
        <option>Option A</option>
        <option>Option B</option>
        <option>Option C</option>
      </select>
    ),
  },
];

function ComponentRow({ selector, preview }: ComponentEntry) {
  return (
    <div className={cs.row}>
      <code className={cs.selector}>{selector}</code>
      <div className={cs.preview}>{preview}</div>
    </div>
  );
}

function ComponentSection({ title, entries }: { title: string; entries: ComponentEntry[] }) {
  return (
    <section className={cs.section}>
      <h3 className={cs.sectionTitle}>{title}</h3>
      {entries.map((entry) => <ComponentRow key={entry.selector} {...entry} />)}
    </section>
  );
}

export function SharedComponents() {
  return (
    <div className={cs.container}>
      <ComponentSection title="Buttons" entries={BUTTON_ENTRIES} />
      <ComponentSection title="Inputs" entries={INPUT_ENTRIES} />
      <ComponentSection title="Boxes / Cards" entries={SURFACE_ENTRIES} />
    </div>
  );
}

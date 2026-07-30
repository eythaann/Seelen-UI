import type { Plugins, Sensors } from "@dnd-kit/abstract";
import {
  DragDropManager,
  // KeyboardSensor,
  PointerActivationConstraints,
  PointerSensor,
} from "@dnd-kit/dom";

function DND_PLUGINS(defaults: Plugins): Plugins {
  return [...defaults];
}

const DND_SENSORS: Sensors = [
  PointerSensor.configure({
    // Allow dragging from buttons and other interactive elements inside items.
    preventActivation: () => false,
    // Prevents unintentional drags on click by requiring a 5px movement before activation.
    activationConstraints: (event, _source) => {
      const { pointerType } = event;
      switch (pointerType) {
        case "mouse":
          return [new PointerActivationConstraints.Distance({ value: 5 })];
        case "touch":
          return [new PointerActivationConstraints.Delay({ value: 250, tolerance: 5 })];
        default:
          return [
            new PointerActivationConstraints.Delay({ value: 200, tolerance: 10 }),
            new PointerActivationConstraints.Distance({ value: 5 }),
          ];
      }
    },
  }),
  /* KeyboardSensor.configure({
    preventActivation: () => true,
  }), */
];

export function createDragDropManager() {
  return new DragDropManager({
    sensors: DND_SENSORS,
    plugins: DND_PLUGINS,
  });
}

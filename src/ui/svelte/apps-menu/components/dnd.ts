import { KeyboardSensor, PointerActivationConstraints, PointerSensor } from "@dnd-kit/dom";

const DND_ACTIVATION_DISTANCE = 24;

export const startMenuDndSensors = [
  PointerSensor.configure({
    preventActivation: () => false,
    activationConstraints: [
      new PointerActivationConstraints.Distance({ value: DND_ACTIVATION_DISTANCE }),
    ],
  }),
  KeyboardSensor,
];

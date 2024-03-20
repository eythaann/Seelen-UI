export interface Item {
  label: string;
  icon: string;
  position: number;
  subItems?: Item[];
  action?: string;
}

export interface RouletteStackItem {
  parentIdx?: number;
  items: Item[];
}

export interface RouletteState {
  stack: RouletteStackItem[];
  rotationStep: number;
}
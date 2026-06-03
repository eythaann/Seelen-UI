import { Widget } from "@seelen-ui/lib";
import type { IconPackEntry } from "@seelen-ui/lib/types";

let entry = $state<IconPackEntry | null>(null);

let bgType = $state<"none" | "solid" | "gradient">("none");
let bgColor = $state("#000000");
let bgGradientStart = $state("#000000");
let bgGradientEnd = $state("#ffffff");
let bgGradientAngle = $state(135);
let overlayImage = $state<HTMLImageElement | null>(null);
let overlayX = $state(0);
let overlayY = $state(0);
let overlayScale = $state(1.0);
let saving = $state(false);
let drawMode = $state(false);
let drawColor = $state("#ff0000");
let brushSize = $state(10);

export function resetState() {
  bgType = "none";
  bgColor = "#000000";
  bgGradientStart = "#000000";
  bgGradientEnd = "#ffffff";
  bgGradientAngle = 135;
  overlayImage = null;
  overlayX = 0;
  overlayY = 0;
  overlayScale = 1.0;
  saving = false;
  drawMode = false;
  drawColor = "#ff0000";
  brushSize = 10;
}

Widget.self.onTrigger(({ customArgs }) => {
  entry = (customArgs?.entry as IconPackEntry) ?? null;
  resetState();
});

$effect.root(() => {
  $effect(() => {
    if (entry) {
      Widget.self.show().then(() => Widget.self.focus());
    } else {
      Widget.self.hide();
    }
  });
});

class EditorState {
  get entry() {
    return entry;
  }
  get bgType() {
    return bgType;
  }
  set bgType(v: "none" | "solid" | "gradient") {
    bgType = v;
    if (entry?.icon) {
      entry.icon.isAproximatelySquare = bgType !== "none";
    }
  }
  get bgColor() {
    return bgColor;
  }
  set bgColor(v: string) {
    bgColor = v;
  }
  get bgGradientStart() {
    return bgGradientStart;
  }
  set bgGradientStart(v: string) {
    bgGradientStart = v;
  }
  get bgGradientEnd() {
    return bgGradientEnd;
  }
  set bgGradientEnd(v: string) {
    bgGradientEnd = v;
  }
  get bgGradientAngle() {
    return bgGradientAngle;
  }
  set bgGradientAngle(v: number) {
    bgGradientAngle = v;
  }
  get isApproximatelySquare() {
    return !!entry?.icon?.isAproximatelySquare;
  }
  set isApproximatelySquare(v: boolean) {
    if (entry?.icon) {
      entry.icon.isAproximatelySquare = v;
    }
  }
  get overlayImage() {
    return overlayImage;
  }
  set overlayImage(v: HTMLImageElement | null) {
    overlayImage = v;
  }
  get overlayX() {
    return overlayX;
  }
  set overlayX(v: number) {
    overlayX = v;
  }
  get overlayY() {
    return overlayY;
  }
  set overlayY(v: number) {
    overlayY = v;
  }
  get overlayScale() {
    return overlayScale;
  }
  set overlayScale(v: number) {
    overlayScale = v;
  }
  get saving() {
    return saving;
  }
  set saving(v: boolean) {
    saving = v;
  }
  get drawMode() {
    return drawMode;
  }
  set drawMode(v: boolean) {
    drawMode = v;
  }
  get drawColor() {
    return drawColor;
  }
  set drawColor(v: string) {
    drawColor = v;
  }
  get brushSize() {
    return brushSize;
  }
  set brushSize(v: number) {
    brushSize = v;
  }
}

export const editorState = new EditorState();

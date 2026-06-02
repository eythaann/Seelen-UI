<script lang="ts">
  import { editorState } from "./state.svelte";

  const CANVAS_SIZE = 256;
  const CANVAS_HALF_SIZE = CANVAS_SIZE / 2;

  let {
    canvas = $bindable<HTMLCanvasElement>(),
    ondrop,
  }: {
    canvas?: HTMLCanvasElement;
    ondrop?: (e: DragEvent) => void;
  } = $props();

  $effect(() => {
    if (!canvas) return;
    const ctx = canvas.getContext("2d")!;
    ctx.clearRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

    if (editorState.bgType === "solid") {
      ctx.fillStyle = editorState.bgColor;
      ctx.fillRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);
    } else if (editorState.bgType === "gradient") {
      const angle = (editorState.bgGradientAngle * Math.PI) / 180;
      const x1 = CANVAS_SIZE / 2 - (Math.cos(angle) * CANVAS_SIZE) / 2;
      const y1 = CANVAS_SIZE / 2 - (Math.sin(angle) * CANVAS_SIZE) / 2;
      const x2 = CANVAS_SIZE / 2 + (Math.cos(angle) * CANVAS_SIZE) / 2;
      const y2 = CANVAS_SIZE / 2 + (Math.sin(angle) * CANVAS_SIZE) / 2;
      const grad = ctx.createLinearGradient(x1, y1, x2, y2);
      grad.addColorStop(0, editorState.bgGradientStart);
      grad.addColorStop(1, editorState.bgGradientEnd);
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);
    }

    if (editorState.overlayImage) {
      const img = editorState.overlayImage;

      const boxSize = CANVAS_SIZE * editorState.overlayScale;
      const aspectRatio = img.naturalWidth / img.naturalHeight;
      const w = aspectRatio >= 1 ? boxSize : boxSize * aspectRatio;
      const h = aspectRatio >= 1 ? boxSize / aspectRatio : boxSize;

      // overlayX/Y is the center of the icon; compute top-left for drawImage
      const drawX = editorState.overlayX + CANVAS_HALF_SIZE - w / 2;
      const drawY = editorState.overlayY + CANVAS_HALF_SIZE - h / 2;

      ctx.imageSmoothingEnabled = true;
      ctx.imageSmoothingQuality = "high";
      ctx.drawImage(img, drawX, drawY, w, h);
    }
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<canvas
  bind:this={canvas}
  width={CANVAS_SIZE}
  height={CANVAS_SIZE}
  class="editor-canvas"
  ondragover={(e) => e.preventDefault()}
  {ondrop}
></canvas>

<style>
  .editor-canvas {
    display: block;
    width: 256px;
    height: 256px;
    flex-shrink: 0;
    border: 1px dashed var(--slu-std-ui-color);
    background: repeating-conic-gradient(#555 0% 25%, #333 0% 50%) 0 0 / 16px 16px;
  }
</style>

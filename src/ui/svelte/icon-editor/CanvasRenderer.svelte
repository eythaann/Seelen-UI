<script lang="ts">
  import { editorState } from "./state.svelte";

  const CANVAS_SIZE = 512;
  const CANVAS_HALF_SIZE = CANVAS_SIZE / 2;

  let {
    bgCanvas = $bindable<HTMLCanvasElement>(),
    imageCanvas = $bindable<HTMLCanvasElement>(),
    drawCanvas = $bindable<HTMLCanvasElement>(),
    ondrop,
  }: {
    bgCanvas?: HTMLCanvasElement;
    imageCanvas?: HTMLCanvasElement;
    drawCanvas?: HTMLCanvasElement;
    ondrop?: (e: DragEvent) => void;
  } = $props();

  let isPointerDown = false;
  let lastX = 0;
  let lastY = 0;

  $effect(() => {
    if (!bgCanvas) return;
    const ctx = bgCanvas.getContext("2d")!;
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
  });

  $effect(() => {
    if (!imageCanvas) return;
    const ctx = imageCanvas.getContext("2d")!;
    ctx.clearRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

    if (editorState.overlayImage) {
      const img = editorState.overlayImage;
      const boxSize = CANVAS_SIZE * editorState.overlayScale;
      const aspectRatio = img.naturalWidth / img.naturalHeight;
      const w = aspectRatio >= 1 ? boxSize : boxSize * aspectRatio;
      const h = aspectRatio >= 1 ? boxSize / aspectRatio : boxSize;
      const drawX = editorState.overlayX + CANVAS_HALF_SIZE - w / 2;
      const drawY = editorState.overlayY + CANVAS_HALF_SIZE - h / 2;
      ctx.imageSmoothingEnabled = true;
      ctx.imageSmoothingQuality = "high";
      ctx.drawImage(img, drawX, drawY, w, h);
    }
  });

  function getCanvasPos(e: PointerEvent): [number, number] {
    if (!drawCanvas) return [0, 0];
    const rect = drawCanvas.getBoundingClientRect();
    const scaleX = CANVAS_SIZE / rect.width;
    const scaleY = CANVAS_SIZE / rect.height;
    return [(e.clientX - rect.left) * scaleX, (e.clientY - rect.top) * scaleY];
  }

  function onPointerDown(e: PointerEvent) {
    if (!editorState.drawMode || !drawCanvas) return;
    e.preventDefault();
    isPointerDown = true;
    drawCanvas.setPointerCapture(e.pointerId);
    [lastX, lastY] = getCanvasPos(e);
    const ctx = drawCanvas.getContext("2d")!;
    ctx.beginPath();
    ctx.arc(lastX, lastY, editorState.brushSize / 2, 0, Math.PI * 2);
    ctx.fillStyle = editorState.drawColor;
    ctx.fill();
  }

  function onPointerMove(e: PointerEvent) {
    if (!isPointerDown || !editorState.drawMode || !drawCanvas) return;
    e.preventDefault();
    const [x, y] = getCanvasPos(e);
    const ctx = drawCanvas.getContext("2d")!;
    ctx.beginPath();
    ctx.moveTo(lastX, lastY);
    ctx.lineTo(x, y);
    ctx.strokeStyle = editorState.drawColor;
    ctx.lineWidth = editorState.brushSize;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.stroke();
    [lastX, lastY] = [x, y];
  }

  function onPointerUp() {
    isPointerDown = false;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="canvas-wrapper"
  ondragover={(e) => e.preventDefault()}
  {ondrop}
>
  <canvas bind:this={bgCanvas} width={CANVAS_SIZE} height={CANVAS_SIZE} class="layer"></canvas>
  <canvas bind:this={imageCanvas} width={CANVAS_SIZE} height={CANVAS_SIZE} class="layer"></canvas>
  <canvas
    bind:this={drawCanvas}
    width={CANVAS_SIZE}
    height={CANVAS_SIZE}
    class="layer draw-layer"
    class:draw-active={editorState.drawMode}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointerleave={onPointerUp}
  ></canvas>
</div>

<style>
  .canvas-wrapper {
    position: relative;
    width: 256px;
    height: 256px;
    flex-shrink: 0;
    background: repeating-conic-gradient(#555 0% 25%, #333 0% 50%) 0 0 / 16px 16px;
  }

  .layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: block;
  }

  .draw-layer {
    pointer-events: none;
  }

  .draw-layer.draw-active {
    pointer-events: auto;
    cursor: crosshair;
  }
</style>

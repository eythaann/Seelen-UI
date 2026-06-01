<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { t } from "./i18n";
  import { editorState, resetState } from "./state.svelte";
  import CanvasRenderer from "./CanvasRenderer.svelte";

  const CANVAS_SIZE = 256;
  let canvas = $state<HTMLCanvasElement>();
  let fileInput: HTMLInputElement;

  $effect(() => {
    Widget.self.ready();
  });

  function loadImageFromUrl(url: string) {
    const img = new Image();
    img.onload = () => {
      resetState();
      editorState.overlayImage = img;
    };
    img.src = url;
  }

  $effect(() => {
    const entry = editorState.entry;
    if (!entry) return;
    const icon = entry.icon;
    if (!icon) return;
    const path = icon.base ?? icon.light ?? icon.dark;
    if (!path) return;
    // Fetch as blob to avoid tainting the canvas with the asset:// cross-origin URL
    fetch(convertFileSrc(path))
      .then((r) => r.blob())
      .then((blob) => loadImageFromUrl(URL.createObjectURL(blob)));
  });

  function onFileChange(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    loadImageFromUrl(URL.createObjectURL(file));
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    const file = e.dataTransfer?.files?.[0];
    if (!file || !file.type.startsWith("image/")) return;
    loadImageFromUrl(URL.createObjectURL(file));
  }

  async function save() {
    if (!canvas || !editorState.entry || editorState.saving) return;
    editorState.saving = true;
    try {
      const dataUrl = canvas.toDataURL("image/png");
      const iconBase64 = dataUrl.split(",")[1]!;

      await invoke(SeelenCommand.RegisterUserCustomAppIcon, {
        iconBase64,
        entry: editorState.entry,
      });

      Widget.getCurrent().hide();
    } finally {
      editorState.saving = false;
    }
  }
</script>

<div class="icon-editor">
  <div class="editor-main">
    <CanvasRenderer bind:canvas ondrop={onDrop} />

    <div class="controls">
      <section class="control-section">
        <button data-skin="default" onclick={() => fileInput.click()}>
          {$t("pick_image")}
        </button>
        <input
          bind:this={fileInput}
          type="file"
          accept="image/*"
          style="display:none"
          onchange={onFileChange}
        />
      </section>

      {#if editorState.overlayImage}
        <section class="control-section">
          <label for="range-x" class="section-label"
            >{$t("position_x")}: {Math.round(editorState.overlayX)}</label
          >
          <input
            id="range-x"
            data-skin="flat"
            type="range"
            min={-CANVAS_SIZE}
            max={CANVAS_SIZE}
            bind:value={editorState.overlayX}
          />

          <label for="range-y" class="section-label"
            >{$t("position_y")}: {Math.round(editorState.overlayY)}</label
          >
          <input
            id="range-y"
            data-skin="flat"
            type="range"
            min={-CANVAS_SIZE}
            max={CANVAS_SIZE}
            bind:value={editorState.overlayY}
          />

          <label for="range-scale" class="section-label"
            >{$t("scale")}: {editorState.overlayScale.toFixed(2)}</label
          >
          <input
            id="range-scale"
            data-skin="flat"
            type="range"
            min="0.1"
            max="4"
            step="0.01"
            bind:value={editorState.overlayScale}
          />
        </section>
      {/if}

      <section class="control-section">
        <span class="section-label">{$t("background")}</span>
        <div class="radio-group">
          <label>
            <input data-skin="default" type="radio" bind:group={editorState.bgType} value="none" />
            {$t("none")}
          </label>
          <label>
            <input data-skin="default" type="radio" bind:group={editorState.bgType} value="solid" />
            {$t("solid")}
          </label>
          <label>
            <input
              data-skin="default"
              type="radio"
              bind:group={editorState.bgType}
              value="gradient"
            />
            {$t("gradient")}
          </label>
        </div>
        {#if editorState.bgType === "solid"}
          <input data-skin="default" type="color" bind:value={editorState.bgColor} />
        {:else if editorState.bgType === "gradient"}
          <div class="gradient-controls">
            <input data-skin="default" type="color" bind:value={editorState.bgGradientStart} />
            <input data-skin="default" type="color" bind:value={editorState.bgGradientEnd} />

            <label>
              <span class="section-label">{$t("angle")}: {editorState.bgGradientAngle}°</span>
              <input
                data-skin="flat"
                type="range"
                min="0"
                max="360"
                bind:value={editorState.bgGradientAngle}
              />
            </label>
          </div>
        {/if}
      </section>

      <section class="control-section">
        <label class="checkbox-label">
          <input
            data-skin="default"
            type="checkbox"
            bind:checked={editorState.isApproximatelySquare}
          />
          {$t("approximately_square")}
        </label>
      </section>

      <div class="action-buttons">
        <button data-skin="default" onclick={resetState}>{$t("reset")}</button>
        <button
          data-skin="solid"
          onclick={save}
          disabled={!editorState.entry || editorState.saving}
        >
          {editorState.saving ? $t("saving") : $t("save")}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .icon-editor {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    overflow: auto;
  }

  .editor-main {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-m);
    flex: 1;
    min-height: 0;
    align-items: flex-start;
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-s);
    flex: 1;
    min-width: 200px;
  }

  .control-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .section-label {
    font-size: 0.75rem;
    opacity: 0.7;
  }

  .radio-group {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-m);
    font-size: 0.85rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: 0.85rem;
    cursor: pointer;
  }

  .gradient-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-s);
    flex-wrap: wrap;
  }

  .action-buttons {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-s);
    margin-top: auto;
  }
</style>

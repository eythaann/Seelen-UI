<script lang="ts"></script>

<div class="bg-layers">
  {#each Array.from({ length: 10 }, (_, i) => i) as index}
    <div class={`bg-layer bg-layer-${index + 1}`}></div>
  {/each}
</div>

<style>
  :global(.bg-layers) {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
  }

  :global(.bg-layer) {
    width: 100%;
    height: 100%;
    position: absolute;
  }
</style>

<script lang="ts">
  let { text }: { text: string } = $props();
</script>

<p class="section-label">{text}</p>

<style>
  .section-label {
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 400;
    line-height: 1.1;
    letter-spacing: 6.48px;
    text-transform: uppercase;
    color: var(--text-section);
  }
</style>

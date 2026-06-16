<script lang="ts">
  import { BRAND, BRAND_TAGLINE } from "$lib/brand";
</script>

<div class="wordmark">
  <span class="name">{BRAND}</span>
  <span class="tag">{BRAND_TAGLINE}</span>
</div>

<style>
  .wordmark {
    display: inline-flex;
    align-items: flex-start;
    gap: 2px;
    line-height: 1;
  }
  .name {
    font-family: var(--font-serif);
    font-size: 36px;
    font-weight: 400;
    color: var(--text);
  }
  .tag {
    font-family: var(--font-sans);
    font-weight: 300;
    font-size: 12px;
    color: var(--wordmark-tag);
    margin-top: 11px; /* Figma: tagline offset from wordmark top */
  }
</style>

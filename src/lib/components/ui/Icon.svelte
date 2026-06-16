<script module lang="ts">
  // Inlined Iconoir (MIT) regular icons. Names map to nav destinations.
  // Exported from <script module> so other components can `import { type IconName }`.
  export type IconName = "home" | "list" | "page-plus" | "book" | "frame-tool";
</script>

<script lang="ts">
  // currentColor + configurable size.
  let { name, size = 24 }: { name: IconName; size?: number } = $props();
</script>

<svg
  width={size} height={size} viewBox="0 0 24 24"
  fill="none" stroke="currentColor" stroke-width="1.5"
  stroke-linecap="round" stroke-linejoin="round"
  xmlns="http://www.w3.org/2000/svg" aria-hidden="true"
>
  {#if name === "home"}
    <path d="M2 8L11.7317 3.13416C11.9006 3.04971 12.0994 3.0497 12.2683 3.13416L22 8" />
    <path d="M20 11V19C20 20.1046 19.1046 21 18 21H6C4.89543 21 4 20.1046 4 19V11" />
  {:else if name === "list"}
    <path d="M8 6L20 6" />
    <path d="M4 6.01L4.01 5.99889" />
    <path d="M4 12.01L4.01 11.9989" />
    <path d="M4 18.01L4.01 17.9989" />
    <path d="M8 12L20 12" />
    <path d="M8 18L20 18" />
  {:else if name === "page-plus"}
    <path d="M4 12V2.6C4 2.26863 4.26863 2 4.6 2H16.2515C16.4106 2 16.5632 2.06321 16.6757 2.17574L19.8243 5.32426C19.9368 5.43679 20 5.5894 20 5.74853V21.4C20 21.7314 19.7314 22 19.4 22H11" />
    <path d="M16 2V5.4C16 5.73137 16.2686 6 16.6 6H20" />
    <path d="M1.99219 19H4.99219M7.99219 19H4.99219M4.99219 19V16M4.99219 19V22" />
  {:else if name === "book"}
    <path d="M4 19V5C4 3.89543 4.89543 3 6 3H19.4C19.7314 3 20 3.26863 20 3.6V16.7143" />
    <path d="M6 17L20 17" />
    <path d="M6 21L20 21" />
    <path d="M6 21C4.89543 21 4 20.1046 4 19C4 17.8954 4.89543 17 6 17" />
    <path d="M9 7L15 7" />
  {:else if name === "frame-tool"}
    <path d="M2 7H3M2 17H3M21 7H22M21 17H22M17 3V2M7 3V2M17 22V21M7 22V21M18 6.6V17.4C18 17.7314 17.7314 18 17.4 18H6.6C6.26863 18 6 17.7314 6 17.4V6.6C6 6.26863 6.26863 6 6.6 6H17.4C17.7314 6 18 6.26863 18 6.6Z" />
  {/if}
</svg>

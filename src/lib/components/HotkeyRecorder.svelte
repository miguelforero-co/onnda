<script lang="ts">
  // Keydown-capture combo recorder. States per UI-SPEC L158-163:
  //   Idle       → kbd chip showing combo; click → Capturing.
  //   Capturing  → pulsing coral-bordered field, "Presiona la combinación…"; listens keydown.
  //   Validating → reject combos with no modifier → inline hint.
  //   Saved      → back to Idle, call onCommitted(newCombo).
  // GOTCHA (UI-SPEC L163 / PATTERNS): NEVER register mid-capture. The parent calls
  // schedSave(true) only after onCommitted fires. Default combo "Alt+Space".
  let {
    shortcut = $bindable("Alt+Space"),
    onCommitted,
  }: {
    shortcut?: string;
    onCommitted?: (combo: string) => void;
  } = $props();

  let capturing = $state(false);
  let hint = $state("");

  // Map a keydown event → Tauri global-shortcut accelerator string, or null if invalid.
  function captureCombo(e: KeyboardEvent): { combo: string | null; lone: boolean } {
    const mods: string[] = [];
    if (e.metaKey) mods.push("CommandOrControl");
    if (e.ctrlKey && !e.metaKey) mods.push("Control");
    if (e.altKey) mods.push("Alt");
    if (e.shiftKey) mods.push("Shift");

    const key = e.key;
    // Ignore lone modifier presses — wait for a real key.
    if (key === "Meta" || key === "Control" || key === "Alt" || key === "Shift") {
      return { combo: null, lone: true };
    }

    // Normalize the non-modifier key to an accelerator token.
    let token: string;
    if (key === " ") token = "Space";
    else if (key.length === 1) token = key.toUpperCase();
    else token = key; // e.g. "Enter", "ArrowUp", "F5", "Tab"

    if (mods.length === 0) {
      return { combo: null, lone: false }; // needs a modifier
    }
    return { combo: [...mods, token].join("+"), lone: false };
  }

  function start() {
    capturing = true;
    hint = "";
  }

  function cancel() {
    capturing = false;
    hint = "";
  }

  function onKeydown(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    e.stopPropagation();

    if (e.key === "Escape") { cancel(); return; }

    const { combo, lone } = captureCombo(e);
    if (lone) return; // keep waiting for the real key
    if (!combo) {
      // Validating → rejected: no modifier
      hint = "Usa al menos un modificador (⌘/⌥/⌃)";
      return;
    }

    // Saved: commit and return to Idle. Re-registration happens in the parent
    // (schedSave(true)) AFTER capture completes — never mid-capture.
    shortcut = combo;
    capturing = false;
    hint = "";
    onCommitted?.(combo);
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="hk">
  {#if capturing}
    <button class="field capturing" onclick={cancel} type="button">
      Presiona la combinación…
    </button>
  {:else}
    <button class="field" onclick={start} type="button">
      <kbd>{shortcut}</kbd>
    </button>
  {/if}
</div>
{#if hint}<p class="hk-hint">{hint}</p>{/if}

<style>
  .hk { display: flex; justify-content: flex-end; }

  .field {
    background: var(--bg); border: 1px solid var(--line);
    border-radius: 6px; padding: 4px 9px; min-width: 140px;
    text-align: right; cursor: pointer; outline: none;
    transition: border-color .15s;
    font-family: inherit; font-size: 12.5px; color: var(--text);
  }
  .field:hover { border-color: rgba(232,85,53,.4); }

  .field.capturing {
    border-color: var(--coral);
    color: var(--coral);
    text-align: center;
    animation: pulse 1.2s ease-in-out infinite;
  }

  kbd {
    display: inline-block; background: rgba(0,0,0,.06);
    border-radius: 4px; padding: 1px 5px;
    font-size: 11px; font-family: inherit; color: var(--muted);
  }

  .hk-hint {
    font-size: 11px; color: var(--coral);
    text-align: right; padding: 4px 3px 0; line-height: 1.5;
  }

  @keyframes pulse {
    0%, 100% { border-color: rgba(232,85,53,.4); }
    50%      { border-color: rgba(232,85,53,1); }
  }
</style>

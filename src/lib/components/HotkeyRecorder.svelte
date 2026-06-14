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

  // Map a PHYSICAL key (e.code) → Tauri accelerator token. We use e.code, NOT
  // e.key, because on macOS Option+<key> rewrites e.key to the composed glyph
  // (Option+Space → U+00A0 non-breaking space, Option+L → "¬", etc.), which broke
  // capturing Space and letters. e.code is layout/modifier-independent.
  function codeToToken(code: string): string | null {
    if (code === "Space") return "Space";
    const letter = /^Key([A-Z])$/.exec(code); if (letter) return letter[1];
    const digit = /^Digit([0-9])$/.exec(code); if (digit) return digit[1];
    const numpad = /^Numpad([0-9])$/.exec(code); if (numpad) return numpad[1];
    const fkey = /^(F[0-9]{1,2})$/.exec(code); if (fkey) return fkey[1];
    const map: Record<string, string> = {
      Enter: "Enter", NumpadEnter: "Enter", Tab: "Tab", Backspace: "Backspace",
      Delete: "Delete", Home: "Home", End: "End", PageUp: "PageUp", PageDown: "PageDown",
      ArrowUp: "Up", ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right",
      Minus: "-", Equal: "=", BracketLeft: "[", BracketRight: "]", Semicolon: ";",
      Quote: "'", Comma: ",", Period: ".", Slash: "/", Backslash: "\\", Backquote: "`",
    };
    return map[code] ?? null;
  }

  // Map a keydown event → Tauri global-shortcut accelerator string, or null if invalid.
  function captureCombo(e: KeyboardEvent): { combo: string | null; lone: boolean } {
    const mods: string[] = [];
    if (e.metaKey) mods.push("CommandOrControl");
    if (e.ctrlKey && !e.metaKey) mods.push("Control");
    if (e.altKey) mods.push("Alt");
    if (e.shiftKey) mods.push("Shift");

    // Ignore lone modifier presses (by physical code) — wait for a real key.
    if (/^(Meta|Control|Alt|Shift)(Left|Right)$/.test(e.code)) {
      return { combo: null, lone: true };
    }

    const token = codeToToken(e.code);
    if (!token) return { combo: null, lone: true }; // unknown key → keep waiting

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
    background: var(--bg-2); border: 1px solid var(--line);
    border-radius: var(--r-sm); padding: 4px 9px; min-width: 140px;
    text-align: right; cursor: pointer; outline: none;
    transition: border-color .15s, box-shadow .15s, background .15s;
    font-family: inherit; font-size: 12.5px; color: var(--text);
  }
  .field:hover { border-color: var(--line-strong); }

  .field.capturing {
    border-color: var(--accent);
    color: var(--accent);
    text-align: center;
    animation: pulse 1.2s ease-in-out infinite;
  }

  kbd {
    display: inline-block; background: rgba(255,255,255,0.08);
    border: 1px solid var(--line); border-radius: 5px; padding: 1px 6px;
    font-size: 11px; font-family: inherit; color: var(--muted);
  }

  .hk-hint {
    font-size: 11px; color: var(--accent);
    text-align: right; padding: 4px 3px 0; line-height: 1.5;
  }

  @keyframes pulse {
    0%, 100% { border-color: rgba(255,106,61,.45); }
    50%      { border-color: rgba(255,106,61,1); }
  }
</style>

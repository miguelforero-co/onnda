<script lang="ts">
  import { auth } from "$lib/auth.svelte";
  import { subscribe } from "$lib/subscribe";
  import Wordmark from "$lib/components/ui/Wordmark.svelte";

  let { onsuccess }: { onsuccess: () => void | Promise<void> } = $props();

  type Mode = "login" | "signup" | "reset";
  let mode = $state<Mode>("login");

  // Shared
  let email = $state("");
  let password = $state("");
  let error = $state("");
  let loading = $state(false);

  // Signup-only
  let name = $state("");
  let confirmPassword = $state("");
  let wantsNews = $state(false);

  // Reset-only
  let resetEmail = $state("");
  let newPassword = $state("");
  let resetDone = $state(false);

  function clearError() { error = ""; }

  function switchMode(m: Mode) {
    mode = m;
    error = "";
    resetDone = false;
  }

  async function handleLogin() {
    if (!email.trim() || !password) { error = "Fill in all fields."; return; }
    loading = true; error = "";
    try {
      await auth.login(email.trim(), password);
      onsuccess();
    } catch (e) {
      error = typeof e === "string" ? e : "Sign-in failed.";
    } finally { loading = false; }
  }

  async function handleSignup() {
    if (!name.trim() || !email.trim() || !password || !confirmPassword) {
      error = "Fill in all fields."; return;
    }
    if (password !== confirmPassword) { error = "Passwords don't match."; return; }
    if (password.length < 8) { error = "Password must be at least 8 characters."; return; }
    loading = true; error = "";
    try {
      await auth.signup(name.trim(), email.trim(), password);
      if (wantsNews) void subscribe(email.trim(), name.trim()); // fire-and-forget, never awaited
      onsuccess();
    } catch (e) {
      error = typeof e === "string" ? e : "Could not create account.";
    } finally { loading = false; }
  }

  async function handleReset() {
    if (!resetEmail.trim() || !newPassword) { error = "Fill in all fields."; return; }
    if (newPassword.length < 8) { error = "New password must be at least 8 characters."; return; }
    loading = true; error = "";
    try {
      await auth.resetPassword(resetEmail.trim(), newPassword);
      resetDone = true;
    } catch (e) {
      error = typeof e === "string" ? e : "Could not reset password.";
    } finally { loading = false; }
  }
</script>

<div class="auth">
  <div class="auth-inner">
    <header class="auth-head">
      <Wordmark />
    </header>

    {#if mode !== "reset"}
      <div class="seg" role="tablist">
        <button
          class="seg-btn"
          class:active={mode === "signup"}
          role="tab"
          aria-selected={mode === "signup"}
          onclick={() => switchMode("signup")}
        >Create account</button>
        <button
          class="seg-btn"
          class:active={mode === "login"}
          role="tab"
          aria-selected={mode === "login"}
          onclick={() => switchMode("login")}
        >Sign in</button>
      </div>
    {/if}

    {#if mode === "signup"}
      <p class="note">Your account is local to this Mac.</p>

      <div class="form">
        <div class="field">
          <label for="auth-name">Name</label>
          <input id="auth-name" type="text" placeholder="Your name"
            bind:value={name} oninput={clearError} disabled={loading} autocomplete="name" />
        </div>
        <div class="field">
          <label for="auth-email">Email</label>
          <input id="auth-email" type="email" placeholder="you@email.com"
            bind:value={email} oninput={clearError} disabled={loading} autocomplete="email" />
        </div>
        <div class="field">
          <label for="auth-password">Password</label>
          <input id="auth-password" type="password" placeholder="At least 8 characters"
            bind:value={password} oninput={clearError} disabled={loading} autocomplete="new-password" />
        </div>
        <div class="field">
          <label for="auth-confirm">Confirm password</label>
          <input id="auth-confirm" type="password" placeholder="Repeat your password"
            bind:value={confirmPassword} oninput={clearError} disabled={loading} autocomplete="new-password" />
        </div>
        <label class="check">
          <input type="checkbox" bind:checked={wantsNews} disabled={loading} />
          <span>Send me onnda updates</span>
        </label>
      </div>

      {#if error}<p class="error">{error}</p>{/if}

      <div class="foot">
        <button class="btn" onclick={handleSignup} disabled={loading}>
          {loading ? "Creating account…" : "Create account"}
        </button>
      </div>

    {:else if mode === "login"}
      <div class="form">
        <div class="field">
          <label for="login-email">Email</label>
          <input id="login-email" type="email" placeholder="you@email.com"
            bind:value={email} oninput={clearError} disabled={loading} autocomplete="email" />
        </div>
        <div class="field">
          <label for="login-password">Password</label>
          <input id="login-password" type="password" placeholder="Your password"
            bind:value={password} oninput={clearError} disabled={loading} autocomplete="current-password" />
        </div>
      </div>

      {#if error}<p class="error">{error}</p>{/if}

      <div class="foot">
        <button class="btn" onclick={handleLogin} disabled={loading}>
          {loading ? "Signing in…" : "Sign in"}
        </button>
        <button class="link" onclick={() => switchMode("reset")} disabled={loading}>
          Forgot my password
        </button>
      </div>

    {:else}
      <div class="intro">
        <h1>Reset password</h1>
        <p>Enter your email and choose a new password.</p>
      </div>

      {#if resetDone}
        <p class="success">Password updated. You can sign in now.</p>
        <div class="foot">
          <button class="btn" onclick={() => switchMode("login")}>Sign in</button>
        </div>
      {:else}
        <div class="form">
          <div class="field">
            <label for="reset-email">Email</label>
            <input id="reset-email" type="email" placeholder="you@email.com"
              bind:value={resetEmail} oninput={clearError} disabled={loading} autocomplete="email" />
          </div>
          <div class="field">
            <label for="reset-newpw">New password</label>
            <input id="reset-newpw" type="password" placeholder="At least 8 characters"
              bind:value={newPassword} oninput={clearError} disabled={loading} autocomplete="new-password" />
          </div>
        </div>

        {#if error}<p class="error">{error}</p>{/if}

        <div class="foot">
          <button class="btn" onclick={handleReset} disabled={loading}>
            {loading ? "Resetting…" : "Reset password"}
          </button>
          <button class="link" onclick={() => switchMode("login")} disabled={loading}>Back</button>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .auth {
    height: 100vh;
    overflow-y: auto;
    background-color: var(--bg);
    /* notebook dot texture — same as the app content panel */
    background-image: radial-gradient(var(--dot-grid) 1px, transparent 1.5px);
    background-size: var(--dot-pitch) var(--dot-pitch);
    display: flex;
    justify-content: center;
    align-items: flex-start;
  }

  .auth-inner {
    width: 100%;
    max-width: 360px;
    display: flex;
    flex-direction: column;
    gap: var(--s6);
    padding: 64px var(--s6) var(--s8);
  }

  .auth-head { display: flex; justify-content: center; }

  /* Segmented control — monochrome */
  .seg {
    display: flex;
    gap: var(--s1);
    background: var(--inset);
    border-radius: var(--r-nav);
    padding: var(--s1);
  }
  .seg-btn {
    flex: 1;
    padding: 8px 10px;
    border: none;
    border-radius: calc(var(--r-nav) - 2px);
    background: transparent;
    color: var(--text-muted);
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    letter-spacing: -.01em;
    cursor: pointer;
    transition: background .15s, color .15s;
  }
  .seg-btn.active {
    background: var(--nav-active-bg);
    color: var(--nav-active-ink);
    font-weight: 600;
  }

  .note { font-size: 12px; color: var(--text-muted); margin: calc(var(--s4) * -1) 0 0; }

  .intro { display: flex; flex-direction: column; gap: 6px; }
  .intro h1 { font-size: 20px; font-weight: 600; letter-spacing: -.03em; color: var(--text); }
  .intro p { font-size: 13px; color: var(--text-muted); line-height: 1.6; }

  /* Form */
  .form { display: flex; flex-direction: column; gap: var(--s4); }
  .field { display: flex; flex-direction: column; gap: 6px; }
  .field label {
    font-size: 11.5px; font-weight: 500; color: var(--text-muted); letter-spacing: -.01em;
  }
  .field input {
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 10px 12px;
    color: var(--text);
    font-family: var(--font-sans);
    font-size: 13.5px;
    outline: none;
    width: 100%;
    transition: border-color .15s;
  }
  .field input::placeholder { color: var(--text-muted); }
  .field input:focus { border-color: var(--text-section); }
  .field input:disabled { opacity: .5; cursor: not-allowed; }

  .check { display: flex; align-items: center; gap: 8px; cursor: pointer; }
  .check input[type="checkbox"] {
    width: 15px; height: 15px; accent-color: var(--text); cursor: pointer; flex-shrink: 0;
  }
  .check span { font-size: 12.5px; color: var(--text-muted); }

  .error {
    font-size: 12.5px; color: var(--danger); line-height: 1.5;
    padding: 9px 12px; background: var(--inset);
    border-radius: var(--r-nav); border: 1px solid var(--line);
  }
  .success {
    font-size: 12.5px; color: var(--text); line-height: 1.5;
    padding: 10px 12px; background: var(--surface);
    border-radius: var(--r-nav); border: 1px solid var(--line);
  }

  .foot { display: flex; flex-direction: column; gap: var(--s2); }

  /* Primary CTA — onnda monochrome (black on light / light on dark) */
  .btn {
    width: 100%;
    background: var(--nav-active-bg);
    color: var(--nav-active-ink);
    border: none;
    border-radius: var(--r-nav);
    padding: 12px;
    font-family: var(--font-sans);
    font-size: 13.5px;
    font-weight: 600;
    letter-spacing: -.01em;
    cursor: pointer;
    transition: opacity .15s, transform .1s;
  }
  .btn:hover:not(:disabled) { opacity: .9; }
  .btn:active:not(:disabled) { transform: scale(.99); }
  .btn:disabled { opacity: .45; cursor: default; }

  .link {
    width: 100%; background: transparent; color: var(--text-muted); border: none;
    padding: 8px; font-family: var(--font-sans); font-size: 13px; font-weight: 500;
    letter-spacing: -.01em; cursor: pointer; transition: color .15s;
  }
  .link:hover:not(:disabled) { color: var(--text); }
  .link:disabled { opacity: .4; cursor: not-allowed; }
</style>

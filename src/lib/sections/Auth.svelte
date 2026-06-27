<script lang="ts">
  import { auth } from "$lib/auth.svelte";
  import { subscribe } from "$lib/subscribe";

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
    if (!email.trim() || !password) { error = "Completa todos los campos."; return; }
    loading = true; error = "";
    try {
      await auth.login(email.trim(), password);
      onsuccess();
    } catch (e) {
      error = typeof e === "string" ? e : "Error al iniciar sesión.";
    } finally { loading = false; }
  }

  async function handleSignup() {
    if (!name.trim() || !email.trim() || !password || !confirmPassword) {
      error = "Completa todos los campos."; return;
    }
    if (password !== confirmPassword) { error = "Las contraseñas no coinciden."; return; }
    if (password.length < 8) { error = "La contraseña debe tener al menos 8 caracteres."; return; }
    loading = true; error = "";
    try {
      await auth.signup(name.trim(), email.trim(), password);
      if (wantsNews) void subscribe(email.trim(), name.trim()); // fire-and-forget, never awaited
      onsuccess();
    } catch (e) {
      error = typeof e === "string" ? e : "Error al crear la cuenta.";
    } finally { loading = false; }
  }

  async function handleReset() {
    if (!resetEmail.trim() || !newPassword) { error = "Completa todos los campos."; return; }
    if (newPassword.length < 8) { error = "La nueva contraseña debe tener al menos 8 caracteres."; return; }
    loading = true; error = "";
    try {
      await auth.resetPassword(resetEmail.trim(), newPassword);
      resetDone = true;
    } catch (e) {
      error = typeof e === "string" ? e : "Error al restablecer la contraseña.";
    } finally { loading = false; }
  }
</script>

<div class="ob">
  <!-- mode tabs -->
  <div class="auth-tabs">
    <button
      class="auth-tab"
      class:active={mode === "signup"}
      onclick={() => switchMode("signup")}
    >Crear cuenta</button>
    <button
      class="auth-tab"
      class:active={mode === "login" || mode === "reset"}
      onclick={() => switchMode("login")}
    >Iniciar sesión</button>
  </div>

  {#if mode === "signup"}
    <div class="ob-intro">
      <h1>Crear cuenta</h1>
      <p class="local-note">Tu cuenta es local en este Mac.</p>
    </div>

    <div class="auth-form">
      <div class="field">
        <label for="auth-name">Nombre</label>
        <input
          id="auth-name"
          type="text"
          placeholder="Tu nombre"
          bind:value={name}
          oninput={clearError}
          disabled={loading}
          autocomplete="name"
        />
      </div>
      <div class="field">
        <label for="auth-email">Correo electrónico</label>
        <input
          id="auth-email"
          type="email"
          placeholder="tu@correo.com"
          bind:value={email}
          oninput={clearError}
          disabled={loading}
          autocomplete="email"
        />
      </div>
      <div class="field">
        <label for="auth-password">Contraseña</label>
        <input
          id="auth-password"
          type="password"
          placeholder="Mínimo 8 caracteres"
          bind:value={password}
          oninput={clearError}
          disabled={loading}
          autocomplete="new-password"
        />
      </div>
      <div class="field">
        <label for="auth-confirm">Confirmar contraseña</label>
        <input
          id="auth-confirm"
          type="password"
          placeholder="Repite tu contraseña"
          bind:value={confirmPassword}
          oninput={clearError}
          disabled={loading}
          autocomplete="new-password"
        />
      </div>
      <label class="checkbox-row">
        <input type="checkbox" bind:checked={wantsNews} disabled={loading} />
        <span>Quiero novedades de onnda</span>
      </label>
    </div>

    {#if error}
      <p class="auth-error">{error}</p>
    {/if}

    <div class="ob-foot">
      <button class="btn-primary" onclick={handleSignup} disabled={loading}>
        {loading ? "Creando cuenta…" : "Crear cuenta"}
      </button>
    </div>

  {:else if mode === "login"}
    <div class="ob-intro">
      <h1>Iniciar sesión</h1>
    </div>

    <div class="auth-form">
      <div class="field">
        <label for="login-email">Correo electrónico</label>
        <input
          id="login-email"
          type="email"
          placeholder="tu@correo.com"
          bind:value={email}
          oninput={clearError}
          disabled={loading}
          autocomplete="email"
        />
      </div>
      <div class="field">
        <label for="login-password">Contraseña</label>
        <input
          id="login-password"
          type="password"
          placeholder="Tu contraseña"
          bind:value={password}
          oninput={clearError}
          disabled={loading}
          autocomplete="current-password"
        />
      </div>
    </div>

    {#if error}
      <p class="auth-error">{error}</p>
    {/if}

    <div class="ob-foot">
      <button class="btn-primary" onclick={handleLogin} disabled={loading}>
        {loading ? "Iniciando sesión…" : "Iniciar sesión"}
      </button>
      <button class="btn-secondary" onclick={() => switchMode("reset")} disabled={loading}>
        Olvidé mi contraseña
      </button>
    </div>

  {:else}
    <!-- reset mode -->
    <div class="ob-intro">
      <h1>Restablecer contraseña</h1>
      <p>Ingresa tu correo y elige una nueva contraseña.</p>
    </div>

    {#if resetDone}
      <div class="auth-success">
        Contraseña actualizada. Ya puedes iniciar sesión.
      </div>
      <div class="ob-foot">
        <button class="btn-primary" onclick={() => switchMode("login")}>
          Iniciar sesión
        </button>
      </div>
    {:else}
      <div class="auth-form">
        <div class="field">
          <label for="reset-email">Correo electrónico</label>
          <input
            id="reset-email"
            type="email"
            placeholder="tu@correo.com"
            bind:value={resetEmail}
            oninput={clearError}
            disabled={loading}
            autocomplete="email"
          />
        </div>
        <div class="field">
          <label for="reset-newpw">Nueva contraseña</label>
          <input
            id="reset-newpw"
            type="password"
            placeholder="Mínimo 8 caracteres"
            bind:value={newPassword}
            oninput={clearError}
            disabled={loading}
            autocomplete="new-password"
          />
        </div>
      </div>

      {#if error}
        <p class="auth-error">{error}</p>
      {/if}

      <div class="ob-foot">
        <button class="btn-primary" onclick={handleReset} disabled={loading}>
          {loading ? "Restableciendo…" : "Restablecer contraseña"}
        </button>
        <button class="btn-secondary" onclick={() => switchMode("login")} disabled={loading}>
          Volver
        </button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .ob {
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 36px 22px 24px;
    gap: 20px;
  }

  .ob-intro { display: flex; flex-direction: column; gap: 6px; }
  .ob-intro h1 { font-size: 22px; font-weight: 600; letter-spacing: -.03em; color: var(--text); }
  .ob-intro p { font-size: 13px; color: var(--muted); line-height: 1.7; }
  .local-note { font-size: 11.5px; color: var(--faint); }

  /* Tabs */
  .auth-tabs {
    display: flex;
    gap: 4px;
    background: var(--surface, rgba(255,255,255,.05));
    border-radius: var(--r, 10px);
    padding: 3px;
  }
  .auth-tab {
    flex: 1;
    padding: 7px 10px;
    border: none;
    border-radius: calc(var(--r, 10px) - 2px);
    background: transparent;
    color: var(--muted);
    font-size: 12.5px;
    font-weight: 500;
    cursor: pointer;
    letter-spacing: -.01em;
    transition: background .15s, color .15s;
  }
  .auth-tab.active {
    background: var(--nav-active-bg, rgba(255,255,255,.1));
    color: var(--nav-active-ink, var(--text));
    font-weight: 600;
  }

  /* Form */
  .auth-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .field label {
    font-size: 11.5px;
    font-weight: 500;
    color: var(--muted);
    letter-spacing: -.01em;
  }
  .field input {
    background: var(--surface, rgba(255,255,255,.05));
    border: 1px solid var(--line, rgba(255,255,255,.1));
    border-radius: var(--r, 10px);
    padding: 9px 12px;
    color: var(--text);
    font-size: 13.5px;
    outline: none;
    transition: border-color .15s;
    width: 100%;
    box-sizing: border-box;
  }
  .field input::placeholder { color: var(--faint); }
  .field input:focus { border-color: var(--accent); }
  .field input:disabled { opacity: .5; cursor: not-allowed; }

  /* Checkbox */
  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }
  .checkbox-row input[type="checkbox"] {
    width: 15px;
    height: 15px;
    accent-color: var(--accent);
    cursor: pointer;
    flex-shrink: 0;
  }
  .checkbox-row span {
    font-size: 12.5px;
    color: var(--muted);
    line-height: 1.4;
  }

  /* Error / success */
  .auth-error {
    font-size: 12.5px;
    color: var(--danger, #ff4d4f);
    line-height: 1.5;
    padding: 8px 12px;
    background: rgba(255, 77, 79, .1);
    border-radius: var(--r, 10px);
    border: 1px solid rgba(255, 77, 79, .25);
    margin: 0;
  }
  .auth-success {
    font-size: 12.5px;
    color: var(--text);
    line-height: 1.5;
    padding: 10px 14px;
    background: rgba(0, 200, 100, .1);
    border-radius: var(--r, 10px);
    border: 1px solid rgba(0, 200, 100, .25);
  }

  /* Footer / CTAs */
  .ob-foot { display: flex; flex-direction: column; gap: 10px; margin-top: auto; }

  .btn-primary {
    width: 100%; background: var(--accent); color: #fff; border: none;
    border-radius: var(--r); padding: 11px; font-size: 13.5px; font-weight: 600;
    cursor: pointer; letter-spacing: -.01em;
    box-shadow: var(--accent-glow);
    transition: opacity .15s, transform .1s, box-shadow .15s;
  }
  .btn-primary:hover:not(:disabled) { opacity: .92; transform: translateY(-1px); }
  .btn-primary:active:not(:disabled) { transform: scale(.98); }
  .btn-primary:disabled {
    opacity: 1; cursor: default;
    background: rgba(255,255,255,0.06); color: var(--faint);
    box-shadow: inset 0 0 0 1px var(--line);
  }

  .btn-secondary {
    width: 100%; background: transparent; color: var(--muted); border: none;
    border-radius: var(--r); padding: 10px; font-size: 13px; font-weight: 500;
    cursor: pointer; letter-spacing: -.01em;
    transition: color .15s;
  }
  .btn-secondary:hover:not(:disabled) { color: var(--text); }
  .btn-secondary:disabled { opacity: .4; cursor: not-allowed; }
</style>

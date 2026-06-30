# Spec B — Cuentas locales, onboarding y datos por perfil (+ anexo C: captación de emails)

**Fecha:** 2026-06-26
**Estado:** aprobado (brainstorming) — pendiente de plan
**Rama destino:** a definir (feature nueva)
**Relación:** independiente de Spec A (métricas). Incluye el anexo **Spec C** (captación de emails vía Vercel). Resuelve el pendiente **#21** ("Hey," sin nombre).

## Objetivo

Introducir **cuentas locales** (email + contraseña + nombre) con **login obligatorio**, y **separar por perfil** los ajustes, el historial y el diccionario. Todo **local en el dispositivo**, sin backend ni sincronización (excepto el anexo C, que sí usa una función serverless mínima solo para captar emails).

## Tensiones aceptadas explícitamente

El usuario eligió un sistema de cuentas email/contraseña aun sabiendo que, sin backend, esto implica:

1. **No es portátil.** Una cuenta creada en este Mac no existe en otro. Los datos no viajan. Se comunica con copy honesto; el día que haya backend, se migra.
2. **No hay reset por correo.** "Olvidé mi contraseña" hace un **reset local** (define nueva contraseña sin verificación externa). Como los datos no están cifrados por cuenta, no hay frontera de seguridad real que esto rompa.
3. **El login es un *gate*, no seguridad.** En v1 los datos por perfil **no se cifran** entre cuentas. Anotado como mejora futura: cifrado derivado de la contraseña (argon2 → key → cifrar la carpeta del perfil), que traería "perder contraseña = perder datos".

## No-objetivos (YAGNI)

- **No** backend de cuentas, **no** sync, **no** OAuth (Google/Apple) en v1.
- **No** cifrado por perfil en v1.
- **No** roles/permisos.

## Modelo de datos

Nuevo archivo global `accounts.json` en `app_data_dir`:

```jsonc
{
  "accounts": [
    {
      "id": "uuid-v4",
      "email": "user@example.com",
      "name": "Miguel",
      "password_hash": "argon2id$...",   // nunca la contraseña en claro
      "created_at": 1750000000000,
      "provider": "local"                // reservado para futuros métodos
    }
  ],
  "current_account_id": "uuid-v4"         // sesión persistida (auto-login)
}
```

- Hash con **argon2id** (crate `argon2`). Verificación en login.
- `current_account_id` persiste la sesión: al reabrir, auto-login en esa cuenta. `null` → mostrar onboarding/login.

### Datos por perfil

Hoy `settings.json`, `history.json`, `recordings/` y el diccionario viven directo en `app_data_dir`. Pasan a:

```
app_data_dir/
  accounts.json                 # global
  models/                       # global (los modelos se comparten entre perfiles)
  profiles/
    <account_id>/
      settings.json
      history.json
      recordings/
```

- **`models/` queda global** (son archivos grandes; no tiene sentido duplicarlos por perfil).
- El diccionario y los replacements ya viven dentro de `settings.json` (`dictionary`, `replacements`, `learned_corrections`), así que se separan automáticamente al separar settings.
- Las funciones `settings_path`, `history_path`, `recordings_dir` pasan a resolver bajo `profiles/<current_account_id>/`. Se introduce un helper único `profile_dir(app) -> app_data_dir/profiles/<current_account_id>`.

### Migración de la data existente

En el primer arranque tras actualizar:

1. Si existe `app_data_dir/settings.json` (esquema viejo, sin `accounts.json`): se marca como **"data heredada pendiente de reclamar"**.
2. La **primera cuenta que se cree** reclama esa data: se mueven `settings.json`, `history.json`, `recordings/` a `profiles/<nueva_cuenta_id>/`.
3. Así el usuario actual **no pierde** su historial/ajustes/diccionario.
4. Idempotente y atómico (mover, no copiar+borrar a medias).

## Onboarding / flujo de UI

El onboarding gana un **primer paso de autenticación**, antes de los pasos actuales (permisos → modelos):

```
Arranque
  -> ¿current_account_id válido? --sí--> ¿onboarding del perfil hecho? --sí--> app
                                                                       --no--> permisos -> modelos -> app
                                --no--> pantalla Auth (Crear cuenta / Iniciar sesión)
                                          -> crea/inicia sesión
                                          -> (si nueva) reclama data heredada
                                          -> permisos -> modelos -> (Spec A: opt-in métricas) -> app
```

- **Pantalla Auth** (nuevo componente, sistema de diseño onnda flat): dos modos.
  - *Crear cuenta:* nombre, email, contraseña (+ confirmación). Validación: email con formato, contraseña mínima (p. ej. ≥ 8). Checkbox opcional **"Quiero novedades de onnda"** (anexo C). Microcopy: "Tu cuenta es local en este Mac."
  - *Iniciar sesión:* email + contraseña. Link "Olvidé mi contraseña" → reset local.
- **`onboarding_done`** pasa a ser **por perfil** (vive en el `settings.json` del perfil). El gate de auth es global (a nivel `accounts.json`).
- **Cerrar sesión** (en Ajustes): `current_account_id = null` → vuelve a la pantalla Auth. No borra datos.
- **Cambiar de cuenta:** cerrar sesión e iniciar con otra. (Multi-cuenta en el mismo Mac soportado.)
- El **nombre** del perfil alimenta el saludo de Home → arregla **#21** ("Hey, {name}").

## Comandos Rust nuevos

- `list_accounts() -> Vec<AccountPublic>` (sin hash).
- `signup(name, email, password, wants_news: bool) -> AccountPublic` (crea, hashea, reclama data heredada, setea sesión; si `wants_news`, dispara anexo C).
- `login(email, password) -> Result<AccountPublic>`.
- `logout()`.
- `current_account() -> Option<AccountPublic>`.
- `reset_password(email, new_password) -> Result<()>` (reset local).

`AccountPublic` = `{ id, email, name, created_at }` (jamás expone `password_hash`).

## Componentes frontend nuevos

- `src/lib/auth.svelte.ts` — store de sesión (cuenta actual, reactividad para mostrar nombre).
- `src/lib/sections/Auth.svelte` (o `components/`) — pantalla crear/iniciar sesión, en el sistema onnda.
- Sección **Cuenta** en Ajustes: nombre/email, cerrar sesión, cambiar contraseña.
- Home: usa `auth` store para el saludo.

## Anexo — Spec C: captación de emails (Vercel)

Pieza pequeña que se enchufa en el signup. Su único fin es armar lista de lanzamiento; es la **semilla del futuro backend**.

- **Función serverless** `POST /api/subscribe` en Vercel.
  - Body: `{ email, name }`.
  - Valida formato de email, **rate-limit** básico, dedup por email.
  - Persiste en un store de Vercel (KV o Postgres — se decide en el plan). MVP aceptable: Vercel KV.
  - Responde `200` siempre que sea válido; nunca expone datos.
- **App:** en `signup`, si `wants_news == true`, hace `POST` **fire-and-forget** (no bloquea el onboarding; si falla, se ignora silenciosamente).
- **Secretos:** la URL del endpoint puede ser pública; cualquier credencial del store vive en variables de entorno de Vercel, **nunca** en el repo.
- **Privacidad:** solo se envía email + nombre **si el usuario marcó el checkbox**. Explícito y opt-in.

## Manejo de errores

- Contraseña incorrecta / email no registrado: mensaje claro, sin filtrar cuáles emails existen más de lo necesario.
- Migración a medias: operación atómica; si falla, no se corrompe la data heredada (se reintenta al próximo signup).
- Anexo C offline o caído: signup procede igual (la cuenta es local; el email a la lista es best-effort).

## Verificación

- Crear cuenta → la data heredada (historial/ajustes/diccionario actuales) aparece bajo el nuevo perfil; nada se pierde.
- Cerrar sesión y crear segunda cuenta → arranca limpia (su propio settings/history vacío); la primera cuenta conserva lo suyo.
- Login con contraseña incorrecta → rechazado; correcta → entra.
- Reset local → permite nueva contraseña y login.
- Home muestra el nombre real (#21 resuelto).
- Anexo C: con checkbox marcado, llega `{email,name}` al endpoint; sin marcar, no se envía nada.
- `cargo check` + `npm run check` verdes.

## Tareas externas

1. Proyecto/funcion en Vercel + store (KV/Postgres) para el anexo C.

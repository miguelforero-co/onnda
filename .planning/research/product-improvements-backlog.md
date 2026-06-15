# Voz Local — Backlog priorizado de mejoras de producto

> Investigación competitiva del estado del arte de dictado por voz en macOS (2025–2026) y derivación de un backlog accionable.
> Fecha: 2026-06-14

## Resumen ejecutivo

El mercado de dictado por voz en macOS se ha bifurcado entre apps cloud-first con LLM (Wispr Flow, Aqua Voice, Willow) que ganan en post-procesado y edición por voz, y apps local-first (Superwhisper, VoiceInk, Spokenly, BetterDictation) que ganan en privacidad, precio único y latencia vía ANE/Parakeet. Voz Local ya está bien posicionada en el eje local-first, pero le faltan tres palancas que todos los líderes ya tienen: **post-procesado con LLM (puntuación/formateo/muletillas), perfiles por app, y diccionario que auto-aprende**. El espacio vacío más valioso del mercado es combinar **edición por voz en lenguaje natural (estilo Aqua)** con **arquitectura 100% local** — nadie lo ha hecho aún.

---

## Tabla comparativa: Voz Local vs competidores

Leyenda: ✅ tiene · ⚠️ parcial/limitado · ❌ no tiene · ☁️ solo cloud · 🔒 local

| Feature | Voz Local | Wispr Flow | Superwhisper | MacWhisper | VoiceInk | Aqua Voice | Spokenly |
|---|---|---|---|---|---|---|---|
| Push-to-talk | ✅ Alt+Space | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Auto-paste en cursor | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cancelar (Esc) | ✅ | ⚠️ | ✅ | ⚠️ | ✅ | ⚠️ | ⚠️ |
| STT 100% local (offline) | ✅ Whisper | ❌ ☁️ | ✅ 🔒 | ✅ 🔒 | ✅ 🔒 | ❌ ☁️ | ✅ 🔒 |
| Motor Parakeet/ANE | ⏳ roadmap | n/a | ✅ | ✅ (CoreML) | ✅ (FluidAudio) | n/a | ✅ |
| Post-procesado LLM | ❌ | ✅ ☁️ | ✅ ☁️ | ✅ (Ollama 🔒) | ✅ (Ollama 🔒) | ✅ ☁️ | ✅ BYOK |
| Quitar muletillas | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Edición por voz (NL) | ❌ | ✅ Command Mode | ⚠️ Super Mode | ❌ | ⚠️ Assistant | ✅ best-in-class | ⚠️ |
| Command mode ("nueva línea") | ❌ | ⚠️ | ❌ | ❌ | ❌ | ✅ | ❌ |
| Context-awareness (app activa) | ❌ | ✅ | ✅ (Super) | ✅ (prompt) | ⚠️ (OCR) | ✅ | ⚠️ |
| Perfiles por app | ❌ | ⚠️ auto | ✅ auto-switch | ⚠️ prompt | ✅ Power Mode | ⚠️ | ⚠️ |
| Diccionario custom | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Auto-aprende correcciones | ❌ ⏳ roadmap | ✅ | ❌ | ⚠️ | ❌ | ❌ | ❌ |
| Snippets / text-expansion | ❌ | ✅ | ✅ Replacements | ✅ | ✅ Smart Replace | ❌ | ✅ regex |
| Multi-idioma + auto-detect | ⚠️ | ✅ 100+ | ✅ 100+ | ✅ 100+ | ✅ 100+ | ✅ 49 | ✅ 100+ |
| Traducción | ❌ | ⚠️ | ✅ | ✅ DeepL | ❌ | ✅ | ❌ |
| Transcripción de archivos | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ |
| Transcripción de reuniones | ❌ | ❌ | ✅ + diarización | ✅ + diarización | ❌ | ❌ | ❌ |
| Historial searchable | ⚠️ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| Estadísticas de uso | ❌ | ✅ (WPM) | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ❌ |
| Widget visualizer | ✅ WebGL notch | ⚠️ bubble | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ⚠️ |
| Pausar multimedia al dictar | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Sonidos de cue | ✅ | ⚠️ | ✅ | ⚠️ | ✅ | ⚠️ | ✅ |
| Onboarding guiado | ⚠️ | ✅ | ✅ | ⚠️ | ⚠️ | ✅ | ⚠️ |
| Integración IDEs / MCP | ❌ | ⚠️ tags | ✅ deep links | ✅ CLI/webhooks | ❌ | ⚠️ | ✅ MCP único |
| Precio | gratis/propio | $12-15/mo | $8.49/mo o $250 único | €59 único | $25-49 único / OSS | $8/mo | gratis local / $10/mo |

**Lecturas clave de la tabla:**
- Voz Local **lidera** en dos cosas que nadie más tiene: *pausar multimedia al dictar* y *visualizer WebGL en el notch*. Son diferenciadores de UX que vale la pena destacar en marketing.
- Las tres brechas más universales (todos los rivales las tienen, Voz Local no): **post-procesado LLM**, **snippets/text-expansion**, **perfiles por app**.
- La brecha más valiosa-y-vacía del mercado: **edición por voz en lenguaje natural hecha 100% local**.

---

## Backlog priorizado (ordenado por valor/esfuerzo — quick wins primero)

Esfuerzo: S (días) · M (1-2 semanas) · L (3+ semanas). Valor: Alto/Medio/Bajo.

### Quick wins (Alto valor, esfuerzo S-M)

| # | Mejora | Descripción | Valor | Esf. | Categoría |
|---|---|---|---|---|---|
| 1 | **Post-procesado LLM local (puntuación + muletillas)** | Pasar el texto crudo por un LLM local (Apple Foundation Models en macOS 26, u Ollama) para puntuar, capitalizar y quitar "este…/o sea/um". Solo texto, audio nunca sale. Es la brecha #1 vs todos los rivales. | Alto | M | Post-procesado |
| 2 | **Snippets / text-expansion por voz** | Decir un disparador ("mi correo", "firma") y expandir a texto predefinido. Reemplazos deterministas case-insensitive sobre el output. Todos los rivales lo tienen; es barato. | Alto | S | UX |
| 3 | **Reemplazos deterministas (find/replace + regex)** | Tabla de correcciones fijas post-transcripción (p.ej. "java script"→"JavaScript"). Complementa el diccionario sin coste de LLM. | Alto | S | Post-procesado |
| 4 | **Estadísticas de uso (WPM, palabras/semana, tiempo ahorrado)** | Panel con métricas tipo Wispr. Engancha y da sensación de progreso con poco código sobre el historial existente. | Medio | S | UX |
| 5 | **Onboarding guiado (permisos + primer dictado)** | Wizard de primera ejecución: micrófono, accesibilidad, atajo, prueba en vivo. Reduce abandono inicial; los líderes invierten mucho aquí. | Alto | S | UX |
| 6 | **Auto-detección de idioma + selector rápido** | Detectar idioma o permitir fijar 2-3 idiomas esperados (mejora precisión). Whisper ya lo soporta; falta exponerlo bien en UI. | Medio | S | Motor-Calidad |
| 7 | **Historial searchable mejorado (full-text + reprocesar)** | Búsqueda full-text sobre transcripciones + botón "procesar de nuevo" (re-aplicar LLM/idioma). Reusar lo que ya hay. | Medio | S | UX |
| 8 | **Auto-learn from corrections** | Cuando el usuario corrige una palabra repetidamente, sugerir añadirla al diccionario/reemplazos. Wispr es el único que lo hace bien; ya está en tu roadmap. | Alto | M | Motor-Calidad |

### Mejoras medias (Alto/Medio valor, esfuerzo M-L)

| # | Mejora | Descripción | Valor | Esf. | Categoría |
|---|---|---|---|---|---|
| 9 | **Motor Parakeet vía ANE/FluidAudio (seleccionable)** | El cuello de latencia es Whisper (~1.7s fijo). Parakeet en ANE da ~190× RTF (sub-segundo). Validado por VoiceInk/Spokenly/Superwhisper. Ya en tu roadmap; alto impacto percibido. | Alto | L | Latencia |
| 10 | **Perfiles por app (modelo + idioma + prompt + auto-enviar)** | Detectar la app activa y cambiar el pipeline: tono formal en Mail, código en IDE, casual en Slack. VoiceInk "Power Mode" es el techo de referencia. | Alto | M | Integración |
| 11 | **Context-awareness vía Accessibility API** | Leer la app y texto cercano vía Accessibility API (no OCR, que es lo que hace mal VoiceInk) para mejorar precisión de nombres propios y tono. Diferenciador real frente a VoiceInk. | Alto | M | Motor-Calidad |
| 12 | **Modos/plantillas configurables (Email, Nota, Mensaje, Código)** | Presets que combinan prompt LLM + idioma + formato. Es el principio organizador de Superwhisper; convierte el LLM en algo controlable por el usuario. | Alto | M | Post-procesado |
| 13 | **Command mode básico ("nueva línea", "borra eso", "punto")** | Comandos hablados para edición simple sin LLM (formateo/navegación). Brecha abierta: ni MacWhisper ni VoiceInk lo tienen. | Medio | M | UX |
| 14 | **Modo manos-libres (toggle) además de push-to-talk** | Activar/desactivar con un atajo en vez de mantener presionado, para dictados largos. Todos los rivales lo ofrecen. | Medio | S | UX |
| 15 | **Wake word opcional ("Oye Voz")** | Activación por voz sin teclas, como "Hey Flow". Útil para accesibilidad y manos ocupadas. | Medio | M | UX |
| 16 | **Traducción on-the-fly (dictar en ES → salida en EN)** | Whisper ya hace translate-to-English; exponerlo y, con LLM local, a más idiomas. MacWhisper lo vende como diferenciador (vía DeepL). | Medio | M | Post-procesado |
| 17 | **Sync de configuración entre dispositivos (diccionario/snippets/modos)** | Sincronizar ajustes (no audio) vía iCloud. Wispr/Superwhisper lo tienen; reduce fricción multi-Mac. | Bajo | M | Integración |
| 18 | **CLI / Shortcuts / deep links** | Exponer dictado como acción de Apple Shortcuts y URL scheme. MacWhisper (`mw` CLI) y Superwhisper lo aprovechan para automatización. | Medio | M | Integración |
| 19 | **Mejoras de accesibilidad (VoiceOver, navegación por teclado, reduced-motion)** | Posicionar Voz Local como herramienta para RSI/movilidad. Wispr lidera aquí; los demás flojean — oportunidad de diferenciación ética y de mercado. | Medio | M | UX |

### Apuestas mayores / menor prioridad inmediata

| # | Mejora | Descripción | Valor | Esf. | Categoría |
|---|---|---|---|---|---|
| 20 | **Transcripción de reuniones (system audio + diarización)** | Capturar audio del sistema + mic, detectar Zoom/Meet, separar hablantes. Superwhisper/MacWhisper lo tienen; FluidAudio ya trae modelos de diarización sin exponer. | Alto | L | Motor-Calidad |
| 21 | **Edición por voz en lenguaje natural (estilo Aqua) — local** | "Quita la última frase", "ponlo en tono formal", "tradúcelo". Pipeline de clasificación de intención (transcribir vs comando vs editar) con LLM local. Es la apuesta diferenciadora. | Alto | L | Post-procesado |
| 22 | **Integración MCP (voz como herramienta para Claude Code/Cursor)** | Exponer Voz Local como servidor MCP para que agentes lo llamen como tool. Único en el mercado (solo Spokenly lo hace). Encaja con tu stack Rust/Tauri. | Medio | L | Integración |
| 23 | **BYOK opcional para LLM cloud (manteniendo default local)** | Permitir clave propia (Claude/OpenAI/Groq) para quien quiera calidad de post-procesado superior, sin romper la promesa local-first (audio nunca sale; solo texto, opt-in). | Medio | M | Post-procesado |

---

## Apuestas grandes (features ambiciosas que diferenciarían el producto)

1. **Edición por voz en lenguaje natural, 100% local.**
   La intersección Aqua (mejor edición por voz) × Spokenly (local-first) está **vacía**. El fundador de Aqua admite públicamente que *no pueden* correr ASR+LLM localmente a la velocidad necesaria — pero Spokenly ya corre Parakeet sub-segundo en ANE. La brecha es solo la capa LLM de edición, no el ASR. Con Parakeet (latencia #9) + un LLM local pequeño/Apple Foundation Models, Voz Local podría ofrecer "habla y edita hablando, sin que nada salga de tu Mac". Sería el primer producto en lograrlo. (#21)

2. **El motor de dictado local más rápido de macOS (Parakeet/ANE + warm-up).**
   Convertir la latencia en bandera de producto: sub-segundo, offline, gratis. Ya tienes el pipeline streaming con warm-up; sumar Parakeet (#9) te pone a la par de Spokenly/Superwhisper y por delante de todo lo cloud en latencia percibida cuando no hay red. (#9)

3. **Perfiles por app inteligentes con context-awareness nativo (Accessibility API).**
   No solo cambiar de modo por app (como VoiceInk), sino leer el contexto real de la app vía Accessibility API (no OCR) para adaptar vocabulario, tono y formato. Combinado con modos configurables (#12) y perfiles (#10), sería un context-awareness más preciso que el de VoiceInk y más controlable que el opaco de Wispr. (#10 + #11 + #12)

4. **Suite de reuniones local con diarización (sin que el audio salga del Mac).**
   Transcripción de reuniones + separación de hablantes + resumen, todo offline. Es la única categoría donde la privacidad importa más (conversaciones sensibles) y donde lo cloud tiene desventaja estructural. FluidAudio ya trae los modelos de diarización. (#20)

5. **Plataforma agéntica de voz vía MCP.**
   Exponer Voz Local como herramienta MCP para Claude Code/Cursor/Codex: el dev dicta y el agente recibe la transcripción como tool-call. Encaja con el stack Tauri/Rust y con tu perfil de usuario (developer). Solo Spokenly lo hace hoy; hay espacio para hacerlo mejor e integrarlo con #21 (editar el prompt del agente por voz). (#22)

---

## Fuentes

**Wispr Flow / Superwhisper:**
- https://wisprflow.ai/features · https://wisprflow.ai/pricing
- https://docs.wisprflow.ai/articles/4816967992-how-to-use-command-mode
- https://docs.wisprflow.ai/articles/4678293671-feature-context-awareness
- https://docs.wisprflow.ai/articles/4052411709-teach-flow-your-words-with-the-dictionary
- https://wisprflow.ai/research/supporting-languages · https://en.wikipedia.org/wiki/Wispr_Flow
- https://superwhisper.com/docs/modes/modes.md · https://superwhisper.com/docs/modes/custom.md
- https://superwhisper.com/docs/modes/super.md · https://superwhisper.com/docs/modes/meeting.md
- https://superwhisper.com/docs/models/voice.md · https://superwhisper.com/docs/models/language.md
- https://superwhisper.com/docs/get-started/interface-vocabulary.md · https://superwhisper.com/voice-coding

**MacWhisper / VoiceInk:**
- https://macwhisper-site.vercel.app/release_notes.html
- https://macwhisper.helpscoutdocs.com (articles 14/31/53/57)
- https://tryvoiceink.com/docs (power-mode, enhancements-configuring-models, transcription-models)
- https://github.com/Beingpax/VoiceInk · https://github.com/FluidInference/FluidAudio

**Aqua Voice / Willow / Spokenly / Talon / Voice Control / BetterDictation / VoiceType:**
- https://aquavoice.com/ · https://aquavoice.com/blog/aqua-voice-for-ios · https://aquavoice.com/blog/introducing-avalon
- https://news.ycombinator.com/item?id=39828686 · https://9to5mac.com/2025/08/15/aqua-voice-shows-just-how-good-mac-dictation-could-be-if-apple-just-tried/
- https://willowvoice.com/features · https://help.willowvoice.com/en/articles/13183983-voice-commands-and-automatic-formatting-guide
- https://spokenly.app/dictation-for-mac · https://spokenly.app/pricing
- https://talonvoice.com/docs · https://talon.wiki · https://github.com/facebookresearch/wav2letter
- https://support.apple.com (Voice Control user guide) · https://betterdictation.com · https://voicetype.com

> Nota: blogs tipo getvoibe / bossai / cultofmac son afiliados o competidor-adyacentes; las cifras duras se cruzaron con fuentes oficiales. Varias claims de velocidad/precisión de vendors ("4x más rápido", "95-97%") no están verificadas independientemente.

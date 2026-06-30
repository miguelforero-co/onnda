// Idiomas soportados por el ASR (whisper auto-detecta con "auto"; Apple recibe locale)
export const LANGUAGES = [
  { label: "Auto-detect", value: "auto" },
  { label: "Spanish",     value: "es" },
  { label: "English",     value: "en" },
  { label: "Portuguese",  value: "pt" },
  { label: "French",      value: "fr" },
  { label: "German",      value: "de" },
] as const;

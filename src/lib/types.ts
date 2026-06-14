// Shared types — mirror the Rust structs in src-tauri/src/settings.rs & history.rs.

export interface Settings {
  shortcut: string;
  push_to_talk: boolean;
  selected_language: string;
  selected_model: string;
  autostart: boolean;
  onboarding_done: boolean;
  widget_position: string;
  custom_words: string;
  word_correction_threshold: number;
  sounds_enabled: boolean;
  sound_on_listen: boolean;
  sound_on_stop: boolean;
  sound_on_cancel: boolean;
  pause_media: boolean;
  dictionary: string[];
}

export interface HistoryEntry {
  id: string;
  timestamp_ms: number;
  text: string;
  audio_filename: string | null;
  duration_secs: number;
  source: string; // "dictation" | "file"
  original_filename: string | null;
}

export interface ModelInfo {
  id: string;
  name: string;
  size_mb: number;
  downloaded: boolean;
  coming_soon: boolean;
}

export interface DownloadProgress {
  model_id: string;
  downloaded_mb: number;
  total_mb: number;
  percent: number;
}

export interface UpdateStatus {
  up_to_date: boolean;
  available_version: string | null;
  current_version: string;
  error: string | null;
}

export type View =
  | "onboarding"
  | "home"
  | "transcripciones"
  | "diccionario"
  | "ajustes";

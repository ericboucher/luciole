/**
 * Typed wrapper around Tauri's `invoke`. Centralising the command surface
 * here keeps the React code free of stringly-typed calls.
 */

import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  language: string;
  whisperModel: string;
  ollamaModel: string;
  obsidianVaultPath: string | null;
  shortcutDictate: string;
  shortcutCorrect: string;
  shortcutTranslate: string;
  shortcutRephrase: string;
  shortcutMeeting: string;
  onboardingCompleted: boolean;
}

export interface OnboardingStatus {
  onboardingCompleted: boolean;
  whisperReady: boolean;
  ollamaInstalled: boolean;
  ollamaModelReady: boolean;
  diarizationReady: boolean;
  microphoneGranted: boolean;
  accessibilityGranted: boolean;
}

export interface Note {
  path: string;
  title: string;
  createdAt: string;
}

export interface GlossaryEntry {
  short: string;
  full: string;
}

export interface MeetingResult {
  notePath: string;
  transcriptPath: string;
  speakersPendingNaming: string[];
}

export type TextAction = "correct" | "translate" | "rephrase" | "clean-dictation";

export interface TextActionRequest {
  action: TextAction;
  text: string;
  targetLanguage?: string;
  tone?: "formel" | "informel";
}

export const ipc = {
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (next: Settings) => invoke<void>("update_settings", { next }),
  onboardingStatus: () => invoke<OnboardingStatus>("onboarding_status"),
  completeOnboarding: () => invoke<void>("complete_onboarding"),
  listNotes: () => invoke<Note[]>("list_notes"),
  readNote: (path: string) => invoke<string>("read_note", { path }),
  listGlossary: () => invoke<GlossaryEntry[]>("list_glossary"),
  addGlossaryEntry: (short: string, full: string) =>
    invoke<void>("add_glossary_entry", { short, full }),
  removeGlossaryEntry: (short: string) =>
    invoke<void>("remove_glossary_entry", { short }),
  startMeeting: () => invoke<string>("start_meeting"),
  stopMeeting: () => invoke<MeetingResult>("stop_meeting"),
  runTextAction: (req: TextActionRequest) =>
    invoke<string>("run_text_action", { req }),
  checkOllamaInstalled: () => invoke<boolean>("check_ollama_installed"),
  checkMicrophonePermission: () =>
    invoke<boolean>("check_microphone_permission"),
  checkAccessibilityPermission: () =>
    invoke<boolean>("check_accessibility_permission"),
  requestAccessibilityPrompt: () =>
    invoke<void>("request_accessibility_prompt"),
};

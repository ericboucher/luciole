/**
 * Typed wrapper around Tauri's `invoke`.
 *
 * Note: this UI can be opened in a normal browser during dev (`vite`), where
 * Tauri globals are not injected. Guard those cases so the app can render a
 * helpful message instead of crashing on load.
 */

type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

function hasTauriInvoke(): boolean {
  const w = globalThis as unknown as {
    __TAURI_INTERNALS__?: { invoke?: unknown };
  };
  return typeof w.__TAURI_INTERNALS__?.invoke === "function";
}

let cachedInvoke: InvokeFn | null = null;

async function getInvoke(): Promise<InvokeFn | null> {
  if (cachedInvoke) return cachedInvoke;
  try {
    const mod = await import("@tauri-apps/api/core");
    cachedInvoke = mod.invoke as InvokeFn;
    return cachedInvoke;
  } catch {
    return null;
  }
}

async function safeInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const inv = await getInvoke();
  if (!inv || !hasTauriInvoke()) {
    throw new Error("TAURI_NOT_AVAILABLE");
  }
  return inv<T>(cmd, args);
}

export function isTauriAvailable(): boolean {
  return hasTauriInvoke();
}

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
  getSettings: () => safeInvoke<Settings>("get_settings"),
  updateSettings: (next: Settings) =>
    safeInvoke<void>("update_settings", { next }),
  onboardingStatus: () => safeInvoke<OnboardingStatus>("onboarding_status"),
  completeOnboarding: () => safeInvoke<void>("complete_onboarding"),
  resetOnboarding: () => safeInvoke<void>("reset_onboarding"),
  listNotes: () => safeInvoke<Note[]>("list_notes"),
  readNote: (path: string) => safeInvoke<string>("read_note", { path }),
  listGlossary: () => safeInvoke<GlossaryEntry[]>("list_glossary"),
  addGlossaryEntry: (short: string, full: string) =>
    safeInvoke<void>("add_glossary_entry", { short, full }),
  removeGlossaryEntry: (short: string) =>
    safeInvoke<void>("remove_glossary_entry", { short }),
  startMeeting: () => safeInvoke<string>("start_meeting"),
  stopMeeting: () => safeInvoke<MeetingResult>("stop_meeting"),
  runTextAction: (req: TextActionRequest) =>
    safeInvoke<string>("run_text_action", { req }),
  runTextActionOnSelection: (
    action: TextAction,
    opts?: { targetLanguage?: string; tone?: "formel" | "informel" },
  ) =>
    safeInvoke<string>("run_text_action_on_selection", {
      req: {
        action,
        targetLanguage: opts?.targetLanguage,
        tone: opts?.tone,
      },
    }),
  checkOllamaInstalled: () => safeInvoke<boolean>("check_ollama_installed"),
  checkMicrophonePermission: () =>
    safeInvoke<boolean>("check_microphone_permission"),
  requestMicrophonePrompt: () =>
    safeInvoke<void>("request_microphone_prompt"),
  checkAccessibilityPermission: () =>
    safeInvoke<boolean>("check_accessibility_permission"),
  requestAccessibilityPrompt: () =>
    safeInvoke<void>("request_accessibility_prompt"),
  openSystemSettings: (panel: "microphone" | "accessibility") =>
    safeInvoke<void>("open_system_settings", { panel }),
  currentExePath: () => safeInvoke<string>("current_exe_path"),
  whisperStatus: () =>
    safeInvoke<{
      binaryPath: string | null;
      modelPath: string;
      modelExists: boolean;
    }>("whisper_status"),
  transcribeMicrophoneTest: (seconds?: number) =>
    safeInvoke<string>("transcribe_microphone_test", {
      seconds: seconds ?? 8,
    }),
};

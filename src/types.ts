export type Phase = "off" | "connecting" | "on" | "stopping" | "error";

export type Region = "auto" | "turkey" | "russia" | "other";

export type Language = "en" | "tr" | "ru";

export interface Status {
  phase: Phase;
  step: number;
  total: number;
  profile: string | null;
  error: string | null;
}

export interface Settings {
  region: Region;
  targets: string[];
  custom: string[];
  autostart: boolean;
  language: string;
  profile: string | null;
  engineActive: boolean;
}

export interface Preferences {
  region: Region;
  targets: string[];
  custom: string[];
}

export interface TargetInfo {
  id: string;
  name: string;
}

export interface Snapshot {
  status: Status;
  settings: Settings;
  targets: TargetInfo[];
  platform: string;
  version: string;
  update: UpdateInfo | null;
}

export interface UpdateInfo {
  version: string;
}

export interface Backend {
  init(language: string): Promise<Snapshot>;
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  savePreferences(preferences: Preferences): Promise<Settings>;
  setLanguage(language: Language): Promise<Settings>;
  setAutostart(enabled: boolean): Promise<Settings>;
  quit(): Promise<void>;
  hide(): Promise<void>;
  onStatus(handler: (status: Status) => void): void;
  onSettings(handler: (settings: Settings) => void): void;
  installUpdate(): Promise<void>;
  openDiscord(): Promise<void>;
  openRepository(): Promise<void>;
  onUpdate(handler: (update: UpdateInfo) => void): void;
}

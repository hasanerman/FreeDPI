import pkg from "../../package.json";
import type { Backend, DiagReport, Phase, Settings, Status } from "../types";

const PROFILES = ["-5 --wrong-seq --reverse-frag", "-5 --set-ttl 5", "-5", "--set-ttl 3", "-9"];

function initialStatus(phase: string | null): Status {
  switch (phase as Phase | null) {
    case "connecting":
      return { phase: "connecting", step: 2, total: PROFILES.length, profile: null, error: null };
    case "on":
      return { phase: "on", step: 1, total: PROFILES.length, profile: PROFILES[0], error: null, since: Date.now() - 3725000, port: 18080 };
    case "error":
      return { phase: "error", step: 0, total: 0, profile: null, error: "no_profile" };
    default:
      return { phase: "off", step: 0, total: 0, profile: null, error: null };
  }
}

export function mockBackend(): Backend {
  const params = new URLSearchParams(location.search);
  const fail = params.get("fail") !== null;
  let status = initialStatus(params.get("phase"));
  let settings: Settings = {
    region: "auto",
    targets: ["discord"],
    custom: [],
    selective: false,
    autostart: false,
    language: params.get("lang") ?? "",
    profile: null,
    engineActive: false,
    notifications: true,
    afterVpn: true,
    debug: params.get("debug") !== null,
  };
  const logHandlers: ((line: string) => void)[] = [];
  const logs: string[] = [
    "2026-11-07 12:00:01Z [I] FreeDPI started on windows",
    "2026-11-07 12:00:04Z [I] trying setting 1/9",
    "2026-11-07 12:00:09Z [W] probe https://discord.com/api/v9/gateway failed (site_reset): connection reset",
    "2026-11-07 12:00:09Z [W] setting 1 did not work (5012 ms)",
    "2026-11-07 12:00:10Z [I] trying setting 2/9",
    "2026-11-07 12:00:12Z [I] setting 2 works (1840 ms)",
    "2026-11-07 12:00:12Z [I] status On",
  ];
  const pushLog = (line: string) => {
    logs.push(line);
    logHandlers.forEach((handler) => handler(line));
  };
  const report: DiagReport = {
    sections: [
      {
        id: "network",
        items: [
          { level: "ok", text: "Online, local address 192.x.x.42", hint: null },
          { level: "info", text: "IPv6 is not available", hint: null },
          { level: "ok", text: "No VPN detected", hint: null },
          { level: "ok", text: "No other system proxy", hint: null },
        ],
      },
      {
        id: "dns",
        items: [
          { level: "warn", text: "discord.com: system DNS 195.175.39.40 (14 ms), secure DNS 162.159.137.232", hint: "dns_mismatch" },
          { level: "ok", text: "FreeDPI DNS server 77.88.8.8:1253 answers (61 ms)", hint: null },
        ],
      },
      {
        id: "sites",
        items: [
          { level: "ok", text: "discord.com (through FreeDPI): DNS 12 ms · TCP 31 ms · HTTPS 200 (240 ms)", hint: null },
          { level: "fail", text: "cdn.discordapp.com (through FreeDPI): DNS 9 ms · TCP 28 ms · HTTPS failed after 5004 ms: connection reset", hint: "site_reset" },
        ],
      },
      {
        id: "engine",
        items: [
          { level: "ok", text: "GoodbyeDPI engine is running", hint: null },
          { level: "ok", text: "WinDivert driver is loaded", hint: null },
        ],
      },
      { id: "history", items: [{ level: "info", text: "status: On, setting 2/9", hint: null }] },
    ],
    text: "FreeDPI diagnostics (mock)",
  };
  const statusHandlers: ((status: Status) => void)[] = [];
  const settingsHandlers: ((settings: Settings) => void)[] = [];
  let timer: number | undefined;

  const emit = (next: Status) => {
    status = next;
    statusHandlers.forEach((handler) => handler(status));
  };
  const update = (next: Partial<Settings>) => {
    settings = { ...settings, ...next };
    settingsHandlers.forEach((handler) => handler(settings));
    return settings;
  };

  return {
    async init(language) {
      if (!settings.language) {
        settings.language = language.toLowerCase().startsWith("tr") ? "tr" : "en";
      }
      return {
        status,
        settings,
        targets: [
          { id: "discord", name: "Discord" },
          { id: "youtube", name: "YouTube" },
          { id: "roblox", name: "Roblox" },
          { id: "wattpad", name: "Wattpad" },
        ],
        platform: params.get("platform") ?? "windows",
        version: pkg.version,
        update: params.get("update") !== null ? { version: "0.2.0", notes: "- Faster connection after sleep\n- Diagnostics button" } : null,
      };
    },
    async connect() {
      if (status.phase === "connecting" || status.phase === "on") return;
      let step = 0;
      const tick = () => {
        step += 1;
        if (step > 3) {
          emit(
            fail
              ? { phase: "error", step: 0, total: 0, profile: null, error: "no_profile" }
              : { phase: "on", step: 3, total: PROFILES.length, profile: PROFILES[2], error: null, since: Date.now() },
          );
          return;
        }
        emit({ phase: "connecting", step, total: PROFILES.length, profile: null, error: null });
        timer = window.setTimeout(tick, 900);
      };
      tick();
    },
    async disconnect() {
      window.clearTimeout(timer);
      emit({ ...status, phase: "stopping" });
      timer = window.setTimeout(() => emit(initialStatus(null)), 500);
    },
    async savePreferences(preferences) {
      return update(preferences);
    },
    async setLanguage(language) {
      return update({ language });
    },
    async setAutostart(enabled) {
      return update({ autostart: enabled });
    },
    async setNotifications(enabled) {
      return update({ notifications: enabled });
    },
    async setAfterVpn(enabled) {
      return update({ afterVpn: enabled });
    },
    async setDebug(enabled) {
      if (enabled) pushLog(`${new Date().toISOString().slice(0, 19).replace("T", " ")}Z [I] debug mode on`);
      return update({ debug: enabled });
    },
    async ping() {
      return status.phase === "on" ? 30 + Math.round(Math.random() * 25) : null;
    },
    async runDiagnostics() {
      await new Promise((resolve) => window.setTimeout(resolve, 1200));
      return report;
    },
    async getLogs() {
      return [...logs];
    },
    async clearLogs() {
      logs.length = 0;
    },
    async saveReport() {
      return "Downloads/FreeDPI-report.txt";
    },
    onLog(handler) {
      logHandlers.push(handler);
    },
    async quit() {},
    async hide() {},
    onStatus(handler) {
      statusHandlers.push(handler);
    },
    onSettings(handler) {
      settingsHandlers.push(handler);
    },
    async installUpdate() {
      await new Promise((resolve) => window.setTimeout(resolve, 1500));
    },
    onUpdate() {},
    async openDiscord() {},
    async reportIssue() {
      window.open("https://github.com/hasanerman/FreeDPI/issues/new", "_blank");
    },
    async diagnostics() {
      return `FreeDPI ${pkg.version}\nstatus: ${status.phase}`;
    },
    async openRepository() {
      window.open("https://github.com/hasanerman/FreeDPI", "_blank");
    },
  };
}

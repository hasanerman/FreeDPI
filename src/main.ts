import { createBackend } from "./api";
import { format, languages, resolveLanguage, strings } from "./i18n";
import type { Backend, Language, Preferences, Region, Settings, Snapshot, Status, UpdateInfo } from "./types";

const REGIONS: Region[] = ["auto", "turkey", "russia", "other"];
const CHECK_ICON = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" /></svg>';
const REMOVE_ICON = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 7l10 10M17 7L7 17" /></svg>';
const DOMAIN = /^(?!-)[a-z0-9-]{1,63}(?<!-)(\.(?!-)[a-z0-9-]{1,63}(?<!-))+$/;

const byId = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;

const ui = {
  app: byId("app"),
  power: byId<HTMLButtonElement>("btn-power"),
  title: byId("status-title"),
  detail: byId("status-detail"),
  profile: byId("status-profile"),
  discord: byId<HTMLButtonElement>("btn-discord"),
  autostartLabel: byId("lbl-autostart"),
  autostart: byId<HTMLInputElement>("chk-autostart"),
  openSettings: byId<HTMLButtonElement>("btn-settings"),
  hide: byId<HTMLButtonElement>("btn-hide"),
  back: byId<HTMLButtonElement>("btn-back"),
  sheet: byId("sheet"),
  settingsTitle: byId("lbl-settings"),
  pending: byId("lbl-pending"),
  regionTitle: byId("lbl-region"),
  regionHint: byId("lbl-region-hint"),
  region: byId("region"),
  languageTitle: byId("lbl-language"),
  language: byId("language"),
  source: byId<HTMLButtonElement>("btn-source"),
  sitesTitle: byId("lbl-sites"),
  sitesHint: byId("lbl-sites-hint"),
  siteTiles: byId("site-tiles"),
  advancedTitle: byId("lbl-advanced"),
  selective: byId<HTMLInputElement>("chk-selective"),
  selectiveLabel: byId("lbl-selective"),
  selectiveHint: byId("lbl-selective-hint"),
  addForm: byId<HTMLFormElement>("add-form"),
  addInput: byId<HTMLInputElement>("add-input"),
  addButton: byId<HTMLButtonElement>("btn-add"),
  version: byId("version"),
  quit: byId<HTMLButtonElement>("btn-quit"),
  update: byId("update"),
  updateText: byId("update-text"),
  updateButton: byId<HTMLButtonElement>("btn-update"),
};

let backend: Backend;
let snapshot: Snapshot;
let status: Status;
let settings: Settings;
let update: UpdateInfo | null = null;
let updating = false;
let flashMessage = "";
let flashTimer: number | undefined;

const language = (): Language => resolveLanguage(settings.language || navigator.language);
const text = () => strings(language());

function detailText(): string {
  const s = text();
  if (flashMessage) return flashMessage;
  switch (status.phase) {
    case "off":
      return s.detail.off;
    case "connecting":
      return status.total > 0 ? format(s.detail.trying, { step: status.step, total: status.total }) : s.detail.preparing;
    case "on":
      return s.detail.on;
    case "stopping":
      return s.detail.stopping;
    case "error": {
      const message = s.errors[status.error ?? "engine"] ?? s.errors.engine;
      return status.error === "no_profile" && snapshot.platform === "windows" ? `${message} ${s.kaspersky}` : message;
    }
  }
}

function renderStatus() {
  const s = text();
  ui.app.dataset.phase = status.phase;
  ui.title.textContent = s.phase[status.phase];
  ui.detail.textContent = detailText();
  ui.profile.textContent = status.phase === "on" && status.step > 0 ? format(s.profile, { step: status.step }) : "";

  const action =
    status.phase === "on" ? s.action.disconnect : status.phase === "connecting" ? s.action.cancel : s.action.connect;
  ui.power.title = action;
  ui.power.setAttribute("aria-label", action);
  ui.power.disabled = status.phase === "stopping";
  ui.pending.hidden = !(status.phase === "on" || status.phase === "connecting");

  ui.discord.hidden = !(snapshot.platform === "macos" && status.phase === "on" && settings.targets.includes("discord"));
  ui.discord.textContent = s.openDiscord;
}

function chips<T extends string>(
  container: HTMLElement,
  options: { id: T; label: string }[],
  active: T,
  select: (id: T) => void,
) {
  container.replaceChildren(
    ...options.map((option) => {
      const chip = document.createElement("button");
      chip.type = "button";
      chip.className = "chip";
      chip.textContent = option.label;
      chip.classList.toggle("active", option.id === active);
      chip.setAttribute("aria-pressed", String(option.id === active));
      chip.addEventListener("click", () => {
        if (option.id !== active) select(option.id);
      });
      return chip;
    }),
  );
}

function tile(name: string, active: boolean, trailing: string): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "tile";
  button.classList.toggle("active", active);
  button.setAttribute("aria-pressed", String(active));
  const label = document.createElement("span");
  label.className = "tile-name";
  label.textContent = name;
  const mark = document.createElement("span");
  mark.className = "mark-icon";
  mark.innerHTML = trailing;
  button.append(label, mark);
  return button;
}

function renderSites() {
  const s = text();
  const presets = snapshot.targets.map((target) => {
    const active = settings.targets.includes(target.id);
    const button = tile(target.name, active, CHECK_ICON);
    button.addEventListener("click", () => {
      const targets = active ? settings.targets.filter((id) => id !== target.id) : [...settings.targets, target.id];
      void save({ targets });
    });
    return button;
  });

  const custom = settings.custom.map((domain) => {
    const button = tile(domain, true, REMOVE_ICON);
    button.classList.add("custom");
    button.title = `${s.remove} ${domain}`;
    button.addEventListener("click", () => void save({ custom: settings.custom.filter((d) => d !== domain) }));
    return button;
  });

  ui.siteTiles.replaceChildren(...presets, ...custom);
}

function renderSettings() {
  const s = text();
  document.documentElement.lang = language();
  ui.autostartLabel.textContent = s.autostart;
  ui.autostart.checked = settings.autostart;
  ui.openSettings.title = s.settings;
  ui.openSettings.setAttribute("aria-label", s.settings);
  ui.hide.title = s.hide;
  ui.hide.setAttribute("aria-label", s.hide);
  ui.back.title = s.back;
  ui.back.setAttribute("aria-label", s.back);
  ui.settingsTitle.textContent = s.settings;
  ui.pending.textContent = s.pending;
  ui.regionTitle.textContent = s.region;
  ui.regionHint.textContent = s.regionHint;
  chips(ui.region, REGIONS.map((id) => ({ id, label: s.regions[id] })), settings.region, (region) => void save({ region }));
  ui.languageTitle.textContent = s.language;
  chips(ui.language, languages, language(), (next) => void changeLanguage(next));
  ui.advancedTitle.textContent = s.advanced;
  ui.selectiveLabel.textContent = s.selective;
  ui.selectiveHint.textContent = s.selectiveHint;
  ui.selective.checked = settings.selective;
  ui.sitesTitle.textContent = s.sites;
  ui.sitesHint.textContent = s.sitesHint;
  renderSites();
  ui.addInput.placeholder = s.addPlaceholder;
  ui.addButton.title = s.addSite;
  ui.addButton.setAttribute("aria-label", s.addSite);
  ui.version.textContent = `FreeDPI ${snapshot.version}`;
  ui.source.title = s.source;
  ui.quit.textContent = s.quit;
}

function renderUpdate() {
  const s = text();
  ui.update.hidden = update === null;
  if (!update) return;
  ui.updateText.textContent = updating ? s.updating : format(s.update, { version: update.version });
  ui.updateButton.textContent = s.updateAction;
  ui.updateButton.hidden = updating;
}

function render() {
  renderSettings();
  renderStatus();
  renderUpdate();
}

function flash(message: string) {
  flashMessage = message;
  window.clearTimeout(flashTimer);
  flashTimer = window.setTimeout(() => {
    flashMessage = "";
    renderStatus();
  }, 4000);
  renderStatus();
}

async function save(change: Partial<Preferences>) {
  const preferences: Preferences = {
    region: settings.region,
    targets: settings.targets,
    custom: settings.custom,
    selective: settings.selective,
    ...change,
  };
  settings = await backend.savePreferences(preferences);
  render();
}

async function changeLanguage(next: Language) {
  settings = await backend.setLanguage(next);
  render();
}

async function installUpdate() {
  updating = true;
  renderUpdate();
  try {
    await backend.installUpdate();
  } catch {
    updating = false;
    update = null;
    renderUpdate();
    flash(text().updateError);
  }
}

function normalizeDomain(input: string): string | null {
  const host = input
    .trim()
    .toLowerCase()
    .replace(/^https?:\/\//, "")
    .split(/[/?#:]/)[0]
    .replace(/\.$/, "");
  return DOMAIN.test(host) && host.length <= 253 ? host : null;
}

function openSheet(open: boolean) {
  ui.sheet.classList.toggle("open", open);
  ui.sheet.setAttribute("aria-hidden", String(!open));
  ui.sheet.inert = !open;
  if (!open) ui.addInput.classList.remove("invalid");
}

function bind() {
  ui.power.addEventListener("click", () => {
    if (status.phase === "off" || status.phase === "error") void backend.connect();
    else if (status.phase === "connecting" || status.phase === "on") void backend.disconnect();
  });

  ui.discord.addEventListener("click", async () => {
    ui.discord.disabled = true;
    try {
      await backend.openDiscord();
    } catch {
      flash(text().discordError);
    }
    ui.discord.disabled = false;
  });

  ui.autostart.addEventListener("change", async () => {
    const wanted = ui.autostart.checked;
    try {
      settings = await backend.setAutostart(wanted);
    } catch {
      flash(text().autostartError);
    }
    render();
  });

  ui.source.addEventListener("click", () => void backend.openRepository());
  ui.selective.addEventListener("change", () => void save({ selective: ui.selective.checked }));

  ui.openSettings.addEventListener("click", () => openSheet(true));
  ui.back.addEventListener("click", () => openSheet(false));
  ui.hide.addEventListener("click", () => void backend.hide());
  ui.quit.addEventListener("click", () => void backend.quit());
  ui.updateButton.addEventListener("click", () => void installUpdate());

  ui.addForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const domain = normalizeDomain(ui.addInput.value);
    if (!domain) {
      ui.addInput.classList.add("invalid");
      ui.addInput.title = text().invalidSite;
      return;
    }
    ui.addInput.classList.remove("invalid");
    ui.addInput.value = "";
    if (!settings.custom.includes(domain)) void save({ custom: [...settings.custom, domain] });
  });
  ui.addInput.addEventListener("input", () => ui.addInput.classList.remove("invalid"));

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") openSheet(false);
  });
  document.addEventListener("contextmenu", (event) => {
    if (!(event.target instanceof HTMLInputElement)) event.preventDefault();
  });
}

async function boot() {
  backend = await createBackend();
  snapshot = await backend.init(navigator.language);
  status = snapshot.status;
  settings = snapshot.settings;
  update = snapshot.update;
  document.documentElement.dataset.platform = snapshot.platform;

  backend.onStatus((next) => {
    status = next;
    renderStatus();
  });
  backend.onSettings((next) => {
    settings = next;
    renderSettings();
    renderStatus();
    renderUpdate();
  });
  backend.onUpdate((next) => {
    update = next;
    renderUpdate();
  });

  bind();
  const devParams = new URLSearchParams(location.search);
  openSheet(import.meta.env.DEV && devParams.has("settings"));
  if (import.meta.env.DEV && devParams.has("shot")) document.body.style.width = "380px";
  render();
  document.body.classList.add("ready");
}

void boot();

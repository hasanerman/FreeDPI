import { createBackend } from "./api";
import { format, languages, resolveLanguage, strings } from "./i18n";
import type { Backend, Language, Preferences, Region, Settings, Snapshot, Status, UpdateInfo } from "./types";

const REGIONS: Region[] = ["auto", "turkey", "russia", "other"];
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
  generalTitle: byId("lbl-general"),
  regionLabel: byId("lbl-region"),
  region: byId<HTMLSelectElement>("region"),
  languageLabel: byId("lbl-language"),
  language: byId<HTMLSelectElement>("language"),
  sitesTitle: byId("lbl-sites"),
  sitesHint: byId("lbl-sites-hint"),
  siteRows: byId("site-rows"),
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

function fillSelect<T extends string>(select: HTMLSelectElement, options: { id: T; label: string }[], active: T) {
  select.replaceChildren(
    ...options.map((option) => {
      const element = document.createElement("option");
      element.value = option.id;
      element.textContent = option.label;
      element.selected = option.id === active;
      return element;
    }),
  );
}

function siteRow(name: string, control: HTMLElement): HTMLElement {
  const row = document.createElement("label");
  row.className = "item";
  const label = document.createElement("span");
  label.textContent = name;
  row.append(label, control);
  return row;
}

function renderSites() {
  const s = text();
  const presets = snapshot.targets.map((target) => {
    const toggle = document.createElement("input");
    toggle.type = "checkbox";
    toggle.className = "switch";
    toggle.checked = settings.targets.includes(target.id);
    toggle.addEventListener("change", () => {
      const targets = toggle.checked
        ? [...settings.targets, target.id]
        : settings.targets.filter((id) => id !== target.id);
      void save({ targets });
    });
    return siteRow(target.name, toggle);
  });

  const custom = settings.custom.map((domain) => {
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "remove";
    remove.title = s.remove;
    remove.setAttribute("aria-label", `${s.remove} ${domain}`);
    remove.innerHTML = REMOVE_ICON;
    remove.addEventListener("click", () => void save({ custom: settings.custom.filter((d) => d !== domain) }));
    const row = siteRow(domain, remove);
    row.classList.add("custom");
    return row;
  });

  ui.siteRows.replaceChildren(...presets, ...custom);
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
  ui.generalTitle.textContent = s.general;
  ui.regionLabel.textContent = s.region;
  fillSelect(ui.region, REGIONS.map((id) => ({ id, label: s.regions[id] })), settings.region);
  ui.languageLabel.textContent = s.language;
  fillSelect(ui.language, languages, language());
  ui.sitesTitle.textContent = s.sites;
  ui.sitesHint.textContent = s.sitesHint;
  renderSites();
  ui.addInput.placeholder = s.addPlaceholder;
  ui.addButton.textContent = s.add;
  ui.version.textContent = `FreeDPI ${snapshot.version}`;
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
    ...change,
  };
  settings = await backend.savePreferences(preferences);
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

  ui.region.addEventListener("change", () => void save({ region: ui.region.value as Region }));
  ui.language.addEventListener("change", async () => {
    settings = await backend.setLanguage(ui.language.value as Language);
    render();
  });

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
  openSheet(false);
  render();
  document.body.classList.add("ready");
}

void boot();

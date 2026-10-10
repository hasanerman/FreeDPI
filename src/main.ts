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
  autostartLabel: byId("lbl-autostart"),
  autostart: byId<HTMLInputElement>("chk-autostart"),
  openSettings: byId<HTMLButtonElement>("btn-settings"),
  hide: byId<HTMLButtonElement>("btn-hide"),
  back: byId<HTMLButtonElement>("btn-back"),
  sheet: byId("sheet"),
  settingsTitle: byId("lbl-settings"),
  pending: byId("lbl-pending"),
  regionTitle: byId("lbl-region"),
  region: byId("region"),
  sitesTitle: byId("lbl-sites"),
  sitesHint: byId("lbl-sites-hint"),
  targets: byId("targets"),
  custom: byId("custom"),
  addForm: byId<HTMLFormElement>("add-form"),
  addInput: byId<HTMLInputElement>("add-input"),
  languageTitle: byId("lbl-language"),
  language: byId("language"),
  version: byId("version"),
  quit: byId<HTMLButtonElement>("btn-quit"),
  update: byId("update"),
  updateText: byId("update-text"),
  updateButton: byId<HTMLButtonElement>("btn-update"),
};

let update: UpdateInfo | null = null;
let updating = false;

function renderUpdate() {
  const s = text();
  ui.update.hidden = update === null;
  if (!update) return;
  ui.updateText.textContent = updating ? s.updating : format(s.update, { version: update.version });
  ui.updateButton.textContent = s.updateAction;
  ui.updateButton.hidden = updating;
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

let backend: Backend;
let snapshot: Snapshot;
let status: Status;
let settings: Settings;
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
  ui.profile.textContent = status.phase === "on" && status.profile ? format(s.profile, { profile: status.profile }) : "";

  const action =
    status.phase === "on" ? s.action.disconnect : status.phase === "connecting" ? s.action.cancel : s.action.connect;
  ui.power.title = action;
  ui.power.setAttribute("aria-label", action);
  ui.power.disabled = status.phase === "stopping";
  ui.pending.hidden = !(status.phase === "on" || status.phase === "connecting");
}

function segmented<T extends string>(
  container: HTMLElement,
  options: { id: T; label: string }[],
  active: T,
  select: (id: T) => void,
) {
  container.replaceChildren(
    ...options.map((option) => {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = option.label;
      button.classList.toggle("active", option.id === active);
      button.setAttribute("aria-pressed", String(option.id === active));
      button.addEventListener("click", () => {
        if (option.id !== active) select(option.id);
      });
      return button;
    }),
  );
}

function renderTargets() {
  ui.targets.replaceChildren(
    ...snapshot.targets.map((target) => {
      const row = document.createElement("label");
      row.className = "check";
      const input = document.createElement("input");
      input.type = "checkbox";
      input.checked = settings.targets.includes(target.id);
      input.addEventListener("change", () => {
        const targets = input.checked
          ? [...settings.targets, target.id]
          : settings.targets.filter((id) => id !== target.id);
        void save({ targets });
      });
      const box = document.createElement("span");
      box.className = "box";
      box.innerHTML = CHECK_ICON;
      const name = document.createElement("span");
      name.textContent = target.name;
      row.append(input, box, name);
      return row;
    }),
  );
}

function renderCustom() {
  ui.custom.hidden = settings.custom.length === 0;
  ui.custom.replaceChildren(
    ...settings.custom.map((domain) => {
      const chip = document.createElement("span");
      chip.className = "chip";
      const label = document.createElement("span");
      label.textContent = domain;
      const remove = document.createElement("button");
      remove.type = "button";
      remove.title = text().remove;
      remove.setAttribute("aria-label", `${text().remove} ${domain}`);
      remove.innerHTML = REMOVE_ICON;
      remove.addEventListener("click", () => void save({ custom: settings.custom.filter((d) => d !== domain) }));
      chip.append(label, remove);
      return chip;
    }),
  );
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
  segmented(
    ui.region,
    REGIONS.map((id) => ({ id, label: s.regions[id] })),
    settings.region,
    (region) => void save({ region }),
  );
  ui.sitesTitle.textContent = s.sites;
  ui.sitesHint.textContent = s.sitesHint;
  renderTargets();
  renderCustom();
  ui.addInput.placeholder = s.addPlaceholder;
  ui.languageTitle.textContent = s.language;
  segmented(ui.language, languages, language(), (id) => void changeLanguage(id));
  ui.version.textContent = format(s.version, { version: snapshot.version });
  ui.quit.textContent = s.quit;
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

async function changeLanguage(next: Language) {
  settings = await backend.setLanguage(next);
  render();
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

  ui.autostart.addEventListener("change", async () => {
    const wanted = ui.autostart.checked;
    try {
      settings = await backend.setAutostart(wanted);
    } catch {
      flash(text().autostartError);
    }
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

  backend.onUpdate((next) => {
    update = next;
    renderUpdate();
  });

  backend.onStatus((next) => {
    status = next;
    renderStatus();
  });
  backend.onSettings((next) => {
    settings = next;
    renderSettings();
    renderUpdate();
  });

  bind();
  openSheet(false);
  render();
  document.body.classList.add("ready");
}

void boot();

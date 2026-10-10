import type { Language, Phase, Region } from "./types";

const en = {
  phase: {
    off: "Off",
    connecting: "Connecting",
    on: "Connected",
    stopping: "Disconnecting",
    error: "Not connected",
  } satisfies Record<Phase, string>,
  detail: {
    off: "Tap to connect",
    preparing: "Getting ready…",
    trying: "Trying setting {step} of {total}",
    on: "You can close this window. FreeDPI keeps running in the background.",
    stopping: "Restoring your connection…",
  },
  errors: {
    missing_files: "Engine files are missing. Please reinstall FreeDPI.",
    needs_admin: "FreeDPI needs administrator permission.",
    system_proxy: "Could not change the system proxy settings.",
    no_profile: "No setting worked on this network. Check your internet and try again.",
    engine: "The engine could not start.",
  } as Record<string, string>,
  kaspersky: "If Kaspersky is installed, remove it. It blocks FreeDPI.",
  profile: "Setting: {profile}",
  action: {
    connect: "Connect",
    disconnect: "Disconnect",
    cancel: "Cancel",
  },
  autostart: "Start with system",
  autostartError: "Could not change startup setting.",
  hide: "Minimize to tray",
  settings: "Settings",
  back: "Back",
  pending: "Changes apply the next time you connect.",
  region: "Region",
  regions: {
    auto: "Auto",
    turkey: "Türkiye",
    russia: "Russia",
    other: "Other",
  } satisfies Record<Region, string>,
  sites: "Sites to unblock",
  sitesHint: "FreeDPI looks for a setting that opens all of these.",
  addPlaceholder: "Add a site, e.g. example.com",
  invalidSite: "Enter a valid domain like example.com",
  remove: "Remove",
  language: "Language",
  quit: "Quit FreeDPI",
  version: "Version {version}",
  update: "Version {version} is available",
  updateAction: "Update",
  updating: "Updating…",
  updateError: "Update failed. Try again later.",
};

type Strings = typeof en;

const tr: Strings = {
  phase: {
    off: "Kapalı",
    connecting: "Bağlanıyor",
    on: "Bağlı",
    stopping: "Kapatılıyor",
    error: "Bağlanamadı",
  },
  detail: {
    off: "Bağlanmak için dokun",
    preparing: "Hazırlanıyor…",
    trying: "Ayar {step}/{total} deneniyor",
    on: "Pencereyi kapatabilirsin. FreeDPI arka planda çalışmaya devam eder.",
    stopping: "Bağlantın eski haline getiriliyor…",
  },
  errors: {
    missing_files: "Motor dosyaları eksik. FreeDPI'ı yeniden kur.",
    needs_admin: "FreeDPI yönetici izni istiyor.",
    system_proxy: "Sistem proxy ayarı değiştirilemedi.",
    no_profile: "Bu ağda hiçbir ayar işe yaramadı. İnternetini kontrol edip tekrar dene.",
    engine: "Motor başlatılamadı.",
  },
  kaspersky: "Kaspersky yüklüyse kaldır, FreeDPI'ı engelliyor.",
  profile: "Ayar: {profile}",
  action: {
    connect: "Bağlan",
    disconnect: "Bağlantıyı kes",
    cancel: "İptal",
  },
  autostart: "Sistemle birlikte başlat",
  autostartError: "Başlangıç ayarı değiştirilemedi.",
  hide: "Tepsiye küçült",
  settings: "Ayarlar",
  back: "Geri",
  pending: "Değişiklikler bir sonraki bağlantıda geçerli olur.",
  region: "Bölge",
  regions: {
    auto: "Otomatik",
    turkey: "Türkiye",
    russia: "Rusya",
    other: "Diğer",
  },
  sites: "Açılacak siteler",
  sitesHint: "FreeDPI bunların hepsini açan ayarı arar.",
  addPlaceholder: "Site ekle, ör. example.com",
  invalidSite: "example.com gibi geçerli bir alan adı yaz",
  remove: "Kaldır",
  language: "Dil",
  quit: "FreeDPI'dan çık",
  version: "Sürüm {version}",
  update: "Yeni sürüm hazır: {version}",
  updateAction: "Güncelle",
  updating: "Güncelleniyor…",
  updateError: "Güncelleme başarısız. Sonra tekrar dene.",
};

const dictionaries: Record<Language, Strings> = { en, tr };

export const languages: { id: Language; label: string }[] = [
  { id: "en", label: "English" },
  { id: "tr", label: "Türkçe" },
];

export function resolveLanguage(value: string): Language {
  return value.toLowerCase().startsWith("tr") ? "tr" : "en";
}

export function strings(language: Language): Strings {
  return dictionaries[language];
}

export function format(template: string, values: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (_, key: string) => String(values[key] ?? ""));
}

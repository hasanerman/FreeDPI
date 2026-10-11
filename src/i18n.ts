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
  profile: "Using setting {step}",
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
  regionHint: "Picks the settings tried first. Auto works everywhere.",
  regions: {
    auto: "Auto",
    turkey: "Türkiye",
    russia: "Russia",
    other: "Other",
  } satisfies Record<Region, string>,
  advanced: "Advanced",
  selective: "Only test sites",
  selectiveHint: "Apply the bypass only to the sites below. Everything else stays untouched. Blocked sites that are not in the list will not open.",
  sites: "Test sites",
  sitesHint:
    "FreeDPI works for every site. These are only used to check which setting works on your network.",
  addSite: "Add site",
  addPlaceholder: "example.com",
  invalidSite: "Enter a valid domain like example.com",
  remove: "Remove",
  language: "Language",
  languageHint: "Changes the language of the app and the tray menu.",
  quit: "Quit",
  source: "Source code on GitHub",
  update: "Version {version} is available",
  updateAction: "Update",
  updating: "Updating…",
  updateError: "Update failed. Try again later.",
  openDiscord: "Open Discord with FreeDPI",
  discordError: "Discord could not be opened.",
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
  profile: "{step}. ayar kullanılıyor",
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
  regionHint: "Önce hangi ayarların deneneceğini seçer. Otomatik her yerde çalışır.",
  regions: {
    auto: "Otomatik",
    turkey: "Türkiye",
    russia: "Rusya",
    other: "Diğer",
  },
  advanced: "Gelişmiş",
  selective: "Sadece test sitelerinde çalış",
  selectiveHint: "Engel aşma sadece aşağıdaki sitelere uygulanır, geri kalan internete hiç dokunulmaz. Listede olmayan engelli siteler açılmaz.",
  sites: "Test siteleri",
  sitesHint:
    "FreeDPI bütün sitelerde çalışır. Bunlar sadece ağında hangi ayarın çalıştığını denemek için kullanılır.",
  addSite: "Site ekle",
  addPlaceholder: "example.com",
  invalidSite: "example.com gibi geçerli bir alan adı yaz",
  remove: "Kaldır",
  language: "Dil",
  languageHint: "Uygulamanın ve tepsi menüsünün dilini değiştirir.",
  quit: "Çıkış",
  source: "GitHub'da kaynak kodu",
  update: "Yeni sürüm hazır: {version}",
  updateAction: "Güncelle",
  updating: "Güncelleniyor…",
  updateError: "Güncelleme başarısız. Sonra tekrar dene.",
  openDiscord: "Discord'u FreeDPI ile aç",
  discordError: "Discord açılamadı.",
};

const ru: Strings = {
  phase: {
    off: "Выключено",
    connecting: "Подключение",
    on: "Подключено",
    stopping: "Отключение",
    error: "Не подключено",
  },
  detail: {
    off: "Нажмите, чтобы подключиться",
    preparing: "Подготовка…",
    trying: "Проверка настройки {step} из {total}",
    on: "Окно можно закрыть. FreeDPI продолжит работать в фоне.",
    stopping: "Восстановление соединения…",
  },
  errors: {
    missing_files: "Файлы движка отсутствуют. Переустановите FreeDPI.",
    needs_admin: "FreeDPI нужны права администратора.",
    system_proxy: "Не удалось изменить системный прокси.",
    no_profile: "Ни одна настройка не сработала в этой сети. Проверьте интернет и попробуйте снова.",
    engine: "Не удалось запустить движок.",
  },
  kaspersky: "Если установлен Kaspersky, удалите его: он блокирует FreeDPI.",
  profile: "Используется настройка {step}",
  action: {
    connect: "Подключить",
    disconnect: "Отключить",
    cancel: "Отмена",
  },
  autostart: "Запускать вместе с системой",
  autostartError: "Не удалось изменить автозапуск.",
  hide: "Свернуть в трей",
  settings: "Настройки",
  back: "Назад",
  pending: "Изменения вступят в силу при следующем подключении.",
  region: "Регион",
  regionHint: "Определяет, какие настройки пробовать первыми. «Авто» работает везде.",
  regions: {
    auto: "Авто",
    turkey: "Турция",
    russia: "Россия",
    other: "Другой",
  },
  advanced: "Дополнительно",
  selective: "Только тестовые сайты",
  selectiveHint: "Обход применяется только к сайтам ниже, остальной трафик не затрагивается. Заблокированные сайты не из списка не откроются.",
  sites: "Тестовые сайты",
  sitesHint:
    "FreeDPI работает для всех сайтов. Эти сайты нужны только для проверки, какая настройка работает в вашей сети.",
  addSite: "Добавить сайт",
  addPlaceholder: "example.com",
  invalidSite: "Введите домен, например example.com",
  remove: "Удалить",
  language: "Язык",
  languageHint: "Меняет язык приложения и меню в трее.",
  quit: "Выйти",
  source: "Исходный код на GitHub",
  update: "Доступна версия {version}",
  updateAction: "Обновить",
  updating: "Обновление…",
  updateError: "Не удалось обновить. Попробуйте позже.",
  openDiscord: "Открыть Discord через FreeDPI",
  discordError: "Не удалось открыть Discord.",
};

const dictionaries: Record<Language, Strings> = { en, tr, ru };

export const languages: { id: Language; label: string }[] = [
  { id: "en", label: "English" },
  { id: "tr", label: "Türkçe" },
  { id: "ru", label: "Русский" },
];

export function resolveLanguage(value: string): Language {
  const lower = value.toLowerCase();
  if (lower.startsWith("tr")) return "tr";
  if (lower.startsWith("ru")) return "ru";
  return "en";
}

export function strings(language: Language): Strings {
  return dictionaries[language];
}

export function format(template: string, values: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (_, key: string) => String(values[key] ?? ""));
}

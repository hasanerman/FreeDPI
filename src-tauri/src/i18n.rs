use crate::controller::Phase;

pub struct Labels {
    pub off: &'static str,
    pub connecting: &'static str,
    pub on: &'static str,
    pub stopping: &'static str,
    pub error: &'static str,
    pub connect: &'static str,
    pub disconnect: &'static str,
    pub cancel: &'static str,
    pub open: &'static str,
    pub autostart: &'static str,
    pub quit: &'static str,
    pub lost: &'static str,
    pub restored: &'static str,
    pub failed: &'static str,
    pub connected: &'static str,
    pub disconnected: &'static str,
    pub not_connected: &'static str,
}

const EN: Labels = Labels {
    off: "Off",
    connecting: "Connecting…",
    on: "Connected",
    stopping: "Disconnecting…",
    error: "Not connected",
    connect: "Connect",
    disconnect: "Disconnect",
    cancel: "Cancel",
    open: "Open FreeDPI",
    autostart: "Start with system",
    quit: "Quit",
    lost: "The connection stopped working. Looking for another setting…",
    restored: "Connected again.",
    failed: "Could not reconnect. Open FreeDPI to try again.",
    connected: "Connected. Blocked sites should open now.",
    disconnected: "Disconnected.",
    not_connected: "Could not connect. Open FreeDPI for details.",
};

const TR: Labels = Labels {
    off: "Kapalı",
    connecting: "Bağlanıyor…",
    on: "Bağlı",
    stopping: "Kapatılıyor…",
    error: "Bağlanamadı",
    connect: "Bağlan",
    disconnect: "Bağlantıyı kes",
    cancel: "İptal",
    open: "FreeDPI'ı aç",
    autostart: "Sistemle birlikte başlat",
    quit: "Çıkış",
    lost: "Bağlantı çalışmayı bıraktı. Başka bir ayar aranıyor…",
    restored: "Yeniden bağlandı.",
    failed: "Yeniden bağlanılamadı. Tekrar denemek için FreeDPI'ı aç.",
    connected: "Bağlandı. Engelli siteler artık açılır.",
    disconnected: "Bağlantı kesildi.",
    not_connected: "Bağlanılamadı. Ayrıntılar için FreeDPI'ı aç.",
};

const RU: Labels = Labels {
    off: "Выключено",
    connecting: "Подключение…",
    on: "Подключено",
    stopping: "Отключение…",
    error: "Не подключено",
    connect: "Подключить",
    disconnect: "Отключить",
    cancel: "Отмена",
    open: "Открыть FreeDPI",
    autostart: "Запускать вместе с системой",
    quit: "Выход",
    lost: "Соединение перестало работать. Ищем другую настройку…",
    restored: "Снова подключено.",
    failed: "Не удалось переподключиться. Откройте FreeDPI, чтобы попробовать снова.",
    connected: "Подключено. Заблокированные сайты теперь открываются.",
    disconnected: "Отключено.",
    not_connected: "Не удалось подключиться. Подробности в FreeDPI.",
};

pub fn labels(language: &str) -> &'static Labels {
    match language {
        "tr" => &TR,
        "ru" => &RU,
        _ => &EN,
    }
}

impl Labels {
    pub fn phase(&self, phase: Phase) -> &'static str {
        match phase {
            Phase::Off => self.off,
            Phase::Connecting => self.connecting,
            Phase::On => self.on,
            Phase::Stopping => self.stopping,
            Phase::Error => self.error,
        }
    }

    pub fn action(&self, phase: Phase) -> &'static str {
        match phase {
            Phase::Off | Phase::Error => self.connect,
            Phase::Connecting => self.cancel,
            Phase::On | Phase::Stopping => self.disconnect,
        }
    }
}

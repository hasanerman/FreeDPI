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
};

pub fn labels(language: &str) -> &'static Labels {
    if language == "tr" { &TR } else { &EN }
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

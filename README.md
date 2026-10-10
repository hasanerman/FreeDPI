<p align="center">
  <img src="src-tauri/icons/128x128.png" width="96" alt="FreeDPI" />
</p>

<h1 align="center">FreeDPI</h1>

<p align="center">
  One-click DPI bypass for Windows and macOS.<br />
  Opens Discord and other sites blocked by your internet provider. No VPN, no remote server, no speed loss.
</p>

---

## What it does

Many internet providers block sites with Deep Packet Inspection (DPI). FreeDPI changes the way your computer sends its first packets so the filter can no longer recognize the site, while the traffic still goes directly to the real server.

Press the power button and FreeDPI tries its settings one by one, checks whether your chosen sites actually open, and keeps the first one that works. It remembers that setting for next time.

- **One button.** Finds a working setting for your network automatically.
- **Runs in the background.** Closing the window keeps it running in the tray (menu bar on macOS). Connect, disconnect and quit from the tray menu.
- **Start with system.** Optionally starts at login and connects on its own.
- **Your sites.** Discord, YouTube, Roblox, Wattpad, or any domain you add.
- **Regions.** Settings tuned for Türkiye and Russia, plus a general list.
- **DNS protection.** Lookups bypass your provider's DNS so poisoned answers do not break sites.
- **English and Türkçe.**

FreeDPI is not a VPN. It does not hide your IP address or encrypt your traffic.

## How it works

| | Windows | macOS |
| --- | --- | --- |
| Engine | [GoodbyeDPI](https://github.com/ValdikSS/GoodbyeDPI) with the WinDivert driver | Built-in Rust engine (local proxy) |
| Scope | All apps, transparently | Apps that use the system proxy (browsers, Discord) |
| DNS | Redirected to Yandex DNS on port 1253 | DNS over HTTPS (Cloudflare, Google) |
| Permissions | Administrator (asked once at launch) | May ask for your password to set the system proxy |

The macOS engine splits the TLS ClientHello into several TCP segments or TLS records so the server name is never in one piece. The same engine is tested on every platform in CI.

## Install

Download the latest installer from [Releases](../../releases).

- **Windows 10/11:** run `FreeDPI_x.y.z_x64-setup.exe`. It adds Start menu and desktop shortcuts and an uninstaller.
- **macOS 11+:** open the `.dmg` and drag FreeDPI to Applications.

If FreeDPI cannot connect on Windows and Kaspersky is installed, uninstall Kaspersky. It blocks the WinDivert driver.

## Build from source

Requirements: Node.js 22+, Rust (stable), and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```
npm install
npm run tauri dev
npm run tauri build
```

Debug builds on Windows do not ask for administrator rights, so the engine only starts in a release build or when the terminal runs as administrator.

Run the tests:

```
cargo test --manifest-path src-tauri/Cargo.toml
```

## Project layout

```
src/                     User interface (TypeScript, no framework)
src-tauri/src/
  controller.rs          Connect / disconnect state machine
  catalog.rs             Regions, settings lists and test sites
  probe.rs               Checks whether the chosen sites open
  tray.rs                Tray icon and menu
  autostart.rs           Start with system (Task Scheduler / LaunchAgent)
  engine/driver.rs       Windows engine (GoodbyeDPI)
  engine/proxy/          Cross-platform proxy engine (TLS split, DoH)
  engine/sysproxy.rs     macOS system proxy
src-tauri/resources/     Bundled GoodbyeDPI and WinDivert binaries (Windows)
third_party/             Licenses and checksums of bundled components
```

## License

FreeDPI is licensed under the [GNU General Public License v3.0](LICENSE).

Bundled third-party components keep their own licenses. See [third_party](third_party/README.md). FreeDPI is an independent project and is not affiliated with GoodbyeDPI.

---

## Türkçe

FreeDPI, internet sağlayıcının DPI ile engellediği Discord gibi siteleri tek tuşla açan bir uygulamadır. Windows ve macOS'ta çalışır. VPN değildir, hızını düşürmez.

- Butona bas, FreeDPI ayarları sırayla dener ve seçtiğin sitelerin açıldığı ilk ayarda kalır.
- Pencereyi kapatınca arka planda çalışmaya devam eder. Sağ alttaki simgeden (macOS'ta üst menü çubuğundan) bağlanıp kesebilirsin.
- İstersen bilgisayar açılınca kendiliğinden başlar ve bağlanır.
- Superonline, Türk Telekom ve diğer sağlayıcılar için Türkiye'ye özel ayarlar içerir.

Kaspersky yüklüyse FreeDPI çalışmayabilir. Kaspersky'yi kaldırman gerekir.

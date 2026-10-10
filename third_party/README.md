# Third-party components

FreeDPI ships the following unmodified third-party binaries on Windows. Their licenses are in this folder.

## GoodbyeDPI 0.2.3rc3 (x86_64)

- Source: https://github.com/ValdikSS/GoodbyeDPI
- License: Apache-2.0 (`goodbyedpi/LICENSE-goodbyedpi.txt`), with `getline` and `uthash` under their own licenses
- Location in the repository: `src-tauri/resources/goodbyedpi/`

| File | SHA-256 |
| --- | --- |
| `goodbyedpi.exe` | `8d412b094bb9c137ff25ba9a794d1122ecc84bb776debff6c249723a13cc31cd` |

## WinDivert (x86_64)

- Source: https://github.com/basil00/WinDivert
- License: LGPL-3.0 or GPL-2.0 (`goodbyedpi/LICENSE-windivert.txt`)

| File | SHA-256 |
| --- | --- |
| `WinDivert.dll` | `6110bfa44667405179c3e15e12af1b62037e447ed59b054b19042032995e6c7e` |
| `WinDivert64.sys` | `e69b5ba3f0cd6cfb2983e442636e7f0b342b61b15264b0328317d4559c82cf50` |

To verify on Windows:

```
certutil -hashfile src-tauri\resources\goodbyedpi\goodbyedpi.exe SHA256
```

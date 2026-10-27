# Runtime Windows — eksperimen

Runtime terpisah dari launcher Rust. Source integrasi berada di folder ini;
layanan PS2 diambil dari PS2Recomp commit `e5636981a1c9de368f6b71e819c3dc188dfedbf4`.

## Status

`TheWarriors.exe` berhasil dibangun sebagai PE32+ x86-64 Windows. Seluruh kode
C++ hasil peta Ghidra berhasil dikompilasi dan ditautkan bersama runtime.
Build ini belum diuji pada perangkat Windows dan belum terbukti mencapai menu
atau gameplay. Ini bukan rilis playable.

`main.cpp` memuat `data/SLUS_212.15`, memeriksa titik masuk dan keberadaan fungsi
boot, serta mengaktifkan pencatatan startup. `boot-frame.png` adalah tangkapan
frame diagnostik, bukan indikator bahwa menu game berhasil ditampilkan.

## Build native Windows

Prasyarat: Visual Studio 2022 Build Tools beserta C++/Windows SDK, Git, CMake 3.21+,
source PS2Recomp pada commit di atas, dan hasil rekompilasi game yang cocok.

Script build menghasilkan kode game dari ELF pengembang dan metadata peta fungsi
yang disertakan. Pengguna akhir memakai runtime siap pakai dari ZIP; mereka tidak
memerlukan compiler. Lihat [flow rilis](../docs/WINDOWS_RELEASE_FLOW.md).

Dari root workspace, gunakan Developer PowerShell:

```powershell
pwsh scripts/Build-WindowsRuntime.ps1 -LocalElf "D:\GameDump\SLUS_212.15"
```

Executable berada di `build/windows-game/bin/Release/TheWarriors.exe`.
DLL FFmpeg yang dibutuhkan disalin oleh build. Lisensi dependensi harus ikut
dalam paket distribusi.

Untuk pengujian lokal saja, ekstrak ISO milik sendiri ke folder `data` di samping
executable. Jalankan dari folder executable tersebut agar path relatif cocok.
Versi eksperimen: USA `SLUS_212.15`, SHA-256 executable:
`1a4e1c3891c930146e39ce340ccb02918cfe917032aafcdbbbe6541f75402369`.

Build Release upstream memakai AVX2 pada Windows. Kompatibilitas CPU tanpa AVX2
belum diuji.

## Build silang yang sudah dilakukan

Build lokal memakai Clang 21 dengan frontend MSVC, CRT `14.44.17.14` dari xwin,
serta SDK Windows. Versi CRT dipin: paket STL terbaru pada sysroot launcher
memerlukan Clang 22 dan tidak cocok untuk compiler ini.

`windows-toolchain.cmake`, `windows-lib`, dan `windows-rc` mendukung build silang
tersebut. Wrapper menggunakan komponen Zig untuk librarian/resource compiler.
Workflow GitHub sebaiknya membangun langsung di runner Windows.

Workflow `.github/workflows/windows.yml` menjalankan build native Windows, memakai
input ELF privat yang diverifikasi, dan menerbitkan ZIP eksperimen pada tag versi.
Workflow belum dijalankan di GitHub; repository remote dan secret perlu disiapkan.

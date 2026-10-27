# The Warriors Launcher — Rust, Windows x64

Launcher native tanpa WebView. Pengguna tidak memerlukan compiler.

1. Ekstrak **seluruh** paket rilis, termasuk folder `runtime` dan DLL-nya.
2. Buka `warriors-launcher.exe`, pilih **Pilih ISO**, lalu pilih ISO USA milik sendiri.
3. Tunggu validasi/extraction. Sediakan sekitar 5 GB ruang kosong untuk aset.
4. Klik **Main** untuk mencoba runtime eksperimen.

Hanya executable USA `SLUS_212.15` dengan hash yang cocok diterima. Launcher tidak
mengunduh ISO. File diekstrak ke staging dengan daftar member tetap; instalasi
lama tidak ditimpa saat gagal. Aset/save berada di folder penyimpanan aplikasi
pengguna. Impor ulang versi yang sama memakai instalasi yang sudah ada.

Runtime diambil dari `runtime/TheWarriors.exe` di samping launcher, sedangkan
working directory game mengarah ke instalasi aset. Paket runtime dapat diperbarui
tanpa memindahkan save. Main dipantau sampai proses selesai dan tidak dibuka dua
kali. Menutup launcher tidak menghentikan game. Selama impor, tunggu sampai selesai.

**Status:** paket eksperimen; menu/gameplay belum terverifikasi. ISO yang berhasil
diimpor tidak membuktikan game sudah playable.

## Pengembangan Windows

Gunakan Rust 1.92.0, MSVC Build Tools dan Windows SDK:

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
$env:RUSTFLAGS = "-C target-feature=+crt-static"
cargo build --locked --release --target x86_64-pc-windows-msvc
```

Test ISO nyata bersifat opt-in dan mengekstrak sekitar 5 GB:

```powershell
$env:WARRIORS_TEST_ISO = "D:\Games\The Warriors.iso"
cargo test --locked supported_iso_imports -- --ignored
```

Rilis lengkap dibangun lewat [GitHub Actions](../.github/workflows/windows.yml);
lihat [flow dan pengaturan secret](../docs/WINDOWS_RELEASE_FLOW.md).

Font Anton oleh Vernon Adams, SIL Open Font License (`assets/OFL.txt`). Skyline
merupakan gambar vektor dari kode. Tidak ada aset ISO yang dibundel pada source.

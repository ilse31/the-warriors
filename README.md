# The Warriors — Windows native port experiment

Target: The Warriors PS2 USA (`SLUS_212.15`) menjadi aplikasi Windows native,
dengan launcher Rust. Project ini belum merupakan port yang dapat dimainkan.

## Komponen

- [Launcher Rust](launcher/README.md): impor ISO sendiri, penyimpanan aset,
  tombol Main dan pemantauan proses. Build Windows x64 sudah dibuat.
- [Integrasi runtime](runtime/README.md): hasil recompilation dihubungkan dengan
  runtime PS2Recomp. Build Windows x64 berhasil; menu/gameplay belum terverifikasi.
- [Flow Windows dan GitHub Actions](docs/WINDOWS_RELEASE_FLOW.md): flow
  import ISO sendiri dan workflow penerbitan prerelease otomatis.

## Source dan data game

Repository ditujukan untuk source launcher, integrasi runtime, konfigurasi, dan
script persiapan. Pengguna menyediakan ISO sendiri. ISO, aset game, output kode
hasil recompilation dan catatan eksperimen lokal dikecualikan melalui `.gitignore`.

GitHub Actions membangun paket siap pakai di Windows. Pengguna akhir hanya memilih
ISO untuk ekstraksi aset. CI memerlukan secret `PRIVATE_GAME_ELF_URL` untuk input
ELF pengembang yang diverifikasi; input itu tidak masuk repository atau paket.
Source tersedia di https://github.com/ilse31/the-warriors. Lihat tab Actions untuk
status CI dan flow rilis untuk pengaturan input build privat.

Dependensi PS2Recomp dipin ke commit `e5636981a1c9de368f6b71e819c3dc188dfedbf4`.
Versi executable yang diteliti memiliki SHA-256
`1a4e1c3891c930146e39ce340ccb02918cfe917032aafcdbbbe6541f75402369`.

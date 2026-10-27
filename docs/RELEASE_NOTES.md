# The Warriors — Windows experimental build

Build ini untuk pengujian, belum siap untuk gameplay. Menu dan gameplay Windows
belum terverifikasi. ISO dan aset game tidak disertakan.

1. Ekstrak seluruh ZIP ke folder yang dapat dibaca, termasuk folder `runtime`.
2. Buka `warriors-launcher.exe` dan klik **Pilih ISO**.
3. Pilih ISO The Warriors USA milik sendiri. Hanya executable `SLUS_212.15` dengan
   hash yang didukung diterima. Sediakan sekitar 5 GB ruang disk untuk aset.
4. Tunggu proses validasi dan ekstraksi selesai, kemudian klik **Main** untuk
   mencoba runtime eksperimen. Pengguna tidak perlu memasang compiler.

Aset dan save disimpan di penyimpanan aplikasi pengguna. Instalasi yang sudah
ada tidak ditimpa bila impor gagal. Pengguna menyediakan ISO sendiri; launcher
tidak mengunduh ISO dari internet.

Target: Windows 10/11 x64, CPU AVX2 dan dukungan OpenGL 3.3. Pengujian perangkat
Windows tetap diperlukan. Jika runtime berhenti atau menampilkan layar hitam,
itu belum berarti instalasi ISO salah: port masih dalam pengembangan.

Source launcher dan integrasi runtime tersedia di tag repository rilis ini.
Runtime memakai PS2Recomp (GPL-3.0), raylib (zlib), dan library FFmpeg beserta
dependensinya. Referensi upstream dan commit tercantum di konfigurasi repository.

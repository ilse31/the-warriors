# Windows ISO launcher validation — 2026-10-27

- Unit tests: 9 passed on the development host; one real-ISO test is opt-in.
- Real-ISO test: passed with the provided USA ISO. Full extraction and reimport
  verified, including preservation of a save fixture. This did not execute the game.
- Windows-specific readonly staging cleanup test is included for native Windows CI;
  it was cross-compiled/linted, not executed on a Windows device locally.
- Clippy passed for host and Windows x64 targets with warnings denied.
- Rustfmt passed. Windows x64 Release build passed with static CRT.
- Actionlint passed for `.github/workflows/windows.yml`.
- Both PowerShell scripts passed syntax parsing. Packaging script executed successfully
  against the existing cross-built Windows runtime and the new Windows launcher.
- ZIP CRC, content allowlist, launcher equality and checksum passed. No ISO, PS2 ELF,
  WAD, audio/movie files or generated C++ are included. Remaining DLL imports are
  Windows system libraries; bundled third-party dependencies are present.

Launcher bytes: 6385664
Launcher SHA-256: `2770c4b73918ab59276ec4df93c62d206fc84ca9e095f1bab0f7c1cc759feff8`.
ZIP SHA-256: `d4ddb3c1f5848f346a86d2e53fc5aa9881116c87e692d6e4c0bc5a0dff5ad623`.

GitHub Actions has not run remotely: the repository has no configured GitHub remote
and `PRIVATE_GAME_ELF_URL` has not been configured. Native Windows first-run UI,
Windows tar extraction of a real ISO, and menu/gameplay remain unverified.
No further native macOS game testing was performed.

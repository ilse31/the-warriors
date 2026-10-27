#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Vec2};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    process::Child,
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Duration,
};

const PAPER: Color32 = Color32::from_rgb(238, 229, 207);
const MUTED: Color32 = Color32::from_rgb(165, 158, 148);
const RED: Color32 = Color32::from_rgb(184, 53, 48);
const BACKGROUND: Color32 = Color32::from_rgb(24, 24, 26);
const STORAGE_KEY: &str = "warriors-settings";

#[derive(Default, Deserialize, Serialize)]
struct Settings {
    game_exe: Option<PathBuf>,
    #[serde(default)]
    data_root: Option<PathBuf>,
}

struct Launcher {
    settings: Settings,
    child: Option<Child>,
    message: String,
    error: bool,
    importing: Option<Receiver<Result<PathBuf, String>>>,
}

impl Launcher {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BACKGROUND;
        visuals.override_text_color = Some(PAPER);
        visuals.selection.bg_fill = RED;
        visuals.widgets.active.bg_fill = RED;
        cc.egui_ctx.set_visuals(visuals);
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "anton".into(),
            egui::FontData::from_static(include_bytes!("../assets/Anton-Regular.ttf")).into(),
        );
        fonts
            .families
            .insert(FontFamily::Name("poster".into()), vec!["anton".into()]);
        cc.egui_ctx.set_fonts(fonts);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(12.0, 10.0);
        style.spacing.button_padding = Vec2::new(18.0, 12.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(16.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(16.0));
        cc.egui_ctx.set_style(style);
        let mut settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, STORAGE_KEY))
            .unwrap_or_default();
        settings.game_exe = settings
            .data_root
            .as_ref()
            .and_then(|_| packaged_runtime().ok());
        Self {
            settings,
            child: None,
            message: String::new(),
            error: false,
            importing: None,
        }
    }

    fn set_message(&mut self, text: impl Into<String>, error: bool) {
        self.message = text.into();
        self.error = error;
    }

    fn choose_iso(&mut self, ctx: &egui::Context) {
        if let Err(error) = packaged_runtime() {
            self.set_message(error, true);
            return;
        }
        let Some(path) = rfd::FileDialog::new()
            .set_title("Pilih ISO The Warriors USA")
            .add_filter("ISO PS2", &["iso"])
            .pick_file()
        else {
            return;
        };
        let Some(storage) = eframe::storage_dir("the-warriors-launcher") else {
            self.set_message("Folder penyimpanan pengguna tidak tersedia.", true);
            return;
        };
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = warriors_launcher::install::import_iso(&path, &storage.join("games"));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.importing = Some(rx);
        self.set_message("Memvalidasi ISO dan mengekstrak aset. Sediakan ruang kosong sekitar 5 GB; tunggu hingga selesai.", false);
    }

    fn poll_import(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Some(receiver) = self.importing.as_ref() else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                self.importing = None;
                match result.and_then(|root| packaged_runtime().map(|exe| (root, exe))) {
                    Ok((root, exe)) => {
                        self.settings.data_root = Some(root);
                        self.settings.game_exe = Some(exe);
                        if let Some(storage) = frame.storage_mut() {
                            eframe::set_value(storage, STORAGE_KEY, &self.settings);
                            storage.flush();
                            self.set_message(
                                "ISO siap. Kamu dapat mencoba build eksperimen ini.",
                                false,
                            );
                        } else {
                            self.set_message(
                                "Aset siap, tetapi pengaturan tidak dapat disimpan.",
                                true,
                            );
                        }
                    }
                    Err(error) => self.set_message(error, true),
                }
            }
            Err(TryRecvError::Disconnected) => {
                self.importing = None;
                self.set_message(
                    "Proses impor berhenti. Instalasi sebelumnya tetap tersimpan.",
                    true,
                );
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(250));
                if ctx.input(|i| i.viewport().close_requested()) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
            }
        }
    }

    fn launch(&mut self) {
        if self.child.is_some() || self.importing.is_some() {
            return;
        }
        if let Some(root) = &self.settings.data_root
            && let Err(error) = warriors_launcher::install::verify_installation(root)
        {
            self.set_message(format!("Aset game tidak siap: {error}"), true);
            return;
        }
        let Some(path) = self.settings.game_exe.as_ref() else {
            return;
        };
        let result = std::env::current_exe()
            .map_err(|e| format!("Lokasi launcher tidak dapat dibaca: {e}"))
            .and_then(|launcher| warriors_launcher::prepare_command(path, &launcher))
            .and_then(|mut command| {
                if let Some(root) = &self.settings.data_root {
                    command.current_dir(root);
                }
                command
                    .spawn()
                    .map_err(|e| format!("Game gagal dibuka: {e}"))
            });
        match result {
            Ok(child) => {
                self.child = Some(child);
                self.set_message(
                    "Runtime dibuka. Build eksperimen ini belum siap untuk gameplay.",
                    false,
                );
            }
            Err(error) => self.set_message(error, true),
        }
    }

    fn poll_game(&mut self, ctx: &egui::Context) {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.child = None;
                    if status.success() {
                        self.set_message(
                            "Game telah ditutup. Sampai jumpa di Coney Island.",
                            false,
                        );
                    } else {
                        self.set_message(
                            format!("Game berhenti dengan {status}. Periksa instalasi game."),
                            true,
                        );
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    self.set_message(format!("Status game tidak dapat diperiksa: {error}"), true)
                }
            }
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }
}

impl eframe::App for Launcher {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, &self.settings);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.poll_import(ctx, frame);
        self.poll_game(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND).inner_margin(28))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("W").size(28.0).strong().color(RED));
                        ui.label(RichText::new("The Warriors").size(18.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("Coney Island • 1979").color(MUTED).size(14.0));
                        });
                    });
                    ui.add_space(22.0);
                    let width = ui.available_width();
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 290.0), egui::Sense::hover());
                    draw_poster(ui, rect);
                    ui.add_space(24.0);
                    ui.heading(RichText::new("Kembali ke jalanan.").size(27.0).strong());
                    ui.label(RichText::new("Pilih ISO The Warriors USA milikmu. Aset disiapkan satu kali.").color(MUTED));
                    ui.add_space(12.0);
                    egui::Frame::new()
                        .fill(Color32::from_rgb(35, 35, 37))
                        .inner_margin(18)
                        .corner_radius(8)
                        .show(ui, |ui| {
                            ui.set_width((width - 36.0).max(0.0));
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("ISO dan aset game").strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add_enabled(self.child.is_none() && self.importing.is_none(), egui::Button::new("Pilih ISO")).clicked() {
                                        self.choose_iso(ctx);
                                    }
                                });
                            });
                            let path = self.settings.data_root.as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "Belum ada ISO yang disiapkan.".into());
                            ui.add(egui::Label::new(RichText::new(path).color(MUTED).size(14.0)).wrap());
                        });
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let running = self.child.is_some();
                        let button = egui::Button::new(RichText::new(if running { "Game berjalan" } else { "Main" })
                            .size(20.0).strong().color(Color32::WHITE))
                            .fill(RED).min_size(Vec2::new(190.0, 52.0));
                        if ui.add_enabled(!running && self.importing.is_none() && self.settings.data_root.is_some() && self.settings.game_exe.is_some(), button).clicked() {
                            self.launch();
                        }
                        ui.vertical(|ui| {
                            ui.label(RichText::new(if running { "Sedang dimainkan" }
                                else if self.importing.is_some() { "Menyiapkan aset…" }
                                else if self.settings.data_root.is_some() { "ISO siap" }
                                else { "Siapkan game terlebih dahulu" }).strong().size(14.0));
                            ui.label(RichText::new("Eksperimen 0.1.0").size(13.0).color(MUTED));
                        });
                    });
                    if !self.message.is_empty() {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&self.message).size(14.0)
                            .color(if self.error { Color32::from_rgb(255, 150, 135) } else { PAPER }));
                    }
                    ui.add_space(18.0);
                    ui.label(RichText::new("Hanya mendukung ISO USA SLUS_212.15. Build eksperimen; menu dan gameplay belum siap.")
                        .size(13.0).color(MUTED));
                });
            });
    }
}

fn draw_poster(ui: &egui::Ui, rect: egui::Rect) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 8.0, Color32::from_rgb(101, 28, 34));
    let skyline = Color32::from_rgb(64, 24, 30);
    let w = rect.width();
    for (x, height, width) in [
        (0.53, 0.26, 0.08),
        (0.61, 0.4, 0.08),
        (0.69, 0.32, 0.09),
        (0.78, 0.48, 0.06),
        (0.84, 0.3, 0.08),
        (0.92, 0.37, 0.08),
    ] {
        let r = egui::Rect::from_min_max(
            egui::pos2(rect.left() + x * w, rect.bottom() - height * rect.height()),
            egui::pos2(rect.left() + (x + width) * w, rect.bottom()),
        );
        painter.rect_filled(r, 0.0, skyline);
    }
    let center = egui::pos2(rect.right() - w * 0.17, rect.top() + 115.0);
    painter.circle_stroke(
        center,
        70.0,
        egui::Stroke::new(2.0, Color32::from_rgb(147, 58, 58)),
    );
    for i in 0..10 {
        let angle = i as f32 * std::f32::consts::TAU / 10.0;
        let end = center + Vec2::angled(angle) * 70.0;
        painter.line_segment(
            [center, end],
            egui::Stroke::new(1.0, Color32::from_rgb(147, 58, 58)),
        );
    }
    painter.line_segment(
        [center, center + Vec2::new(-35.0, 155.0)],
        egui::Stroke::new(3.0, skyline),
    );
    painter.line_segment(
        [center, center + Vec2::new(35.0, 155.0)],
        egui::Stroke::new(3.0, skyline),
    );
    painter.text(
        rect.left_top() + Vec2::new(26.0, 22.0),
        egui::Align2::LEFT_TOP,
        "THE",
        FontId::new(46.0, FontFamily::Name("poster".into())),
        PAPER,
    );
    painter.text(
        rect.left_top() + Vec2::new(23.0, 73.0),
        egui::Align2::LEFT_TOP,
        "WARRIORS",
        FontId::new(
            (w * 0.083).clamp(42.0, 76.0),
            FontFamily::Name("poster".into()),
        ),
        PAPER,
    );
    painter.text(
        rect.left_bottom() + Vec2::new(28.0, -30.0),
        egui::Align2::LEFT_BOTTOM,
        "Can you dig it?",
        FontId::proportional(19.0),
        PAPER,
    );
}

fn main() -> eframe::Result {
    eframe::run_native(
        "The Warriors",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([940.0, 760.0])
                .with_min_inner_size([760.0, 620.0])
                .with_app_id("the-warriors-launcher"),
            centered: true,
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Launcher::new(cc)))),
    )
}

fn packaged_runtime() -> Result<PathBuf, String> {
    let launcher = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = launcher
        .parent()
        .ok_or("Folder launcher tidak tersedia.")?
        .join("runtime")
        .join("TheWarriors.exe");
    if !exe.is_file() {
        return Err("Paket runtime tidak lengkap. Ekstrak seluruh ZIP rilis, termasuk folder runtime, lalu coba lagi.".into());
    }
    Ok(exe)
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use eframe::egui::{self, ScrollArea, TextStyle};
use notify::Watcher;
use rand::{distributions::Alphanumeric, Rng};

fn random_file_name() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(10).map(char::from).collect::<String>()
}

fn create_random_file(dir: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let dir = dir.as_ref();
    let name = random_file_name();
    let path = dir.join(name);
    std::fs::File::create(&path)?;
    Ok(path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 500.0]),
        ..Default::default()
    };
    let mut current_dir = std::env::current_dir().unwrap();
    let mut files = vec![];
    let mut deleted = vec![];
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&current_dir, notify::RecursiveMode::Recursive)?;
    let mut events = vec![];
    eframe::run_simple_native("Notify Test", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Notify Test");
            ui.horizontal(|ui| {
                ui.label("Current directory: ");
                ui.monospace(format!("{}", current_dir.display()));
                if ui.button("Change").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        watcher.unwatch(&current_dir).unwrap();
                        current_dir = path;
                        watcher.watch(&current_dir, notify::RecursiveMode::Recursive).unwrap();
                    }
                }
            });
            if ui.button("Create random file").clicked() {
                if let Ok(path) = create_random_file(&current_dir) {
                    files.push(path);
                }
            }
            egui::Grid::new("files")
                .num_columns(2)
                .show(ui, |ui| {
                    for file in &mut files {
                        ui.label(format!("{}", file.file_name().unwrap().to_string_lossy()));
                        ui.horizontal(|ui| {
                            if ui.button("Delete").clicked() {
                                std::fs::remove_file(&file).unwrap();
                                deleted.push(file.clone());
                            }
                            if ui.button("Rename").clicked() {
                                let new_name = random_file_name();
                                std::fs::rename(&file, &new_name).unwrap();
                                file.set_file_name(new_name);
                            }
                        });
                        ui.end_row();
                    }
                });
            files.retain(|file| !deleted.contains(file));
            deleted.clear();
        });

        egui::TopBottomPanel::bottom("log").resizable(false).min_height(300.0).max_height(300.0).show(ctx, |ui| {
            if let Ok(Ok(event)) = rx.try_recv() {
                events.push(event);
            }
            let text_style = TextStyle::Monospace;
            let row_height = ui.text_style_height(&text_style);
            ScrollArea::vertical().stick_to_bottom(true).show_rows(ui, row_height, events.len(), |ui, row_range| {
                for row in events[row_range].iter() {
                    ui.label(format!("{:?}", row));

                }
            });
        });
    }).map_err(|e| e.to_string().into())
}


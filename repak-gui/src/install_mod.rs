use crate::pak_logic::install_mods_in_viewport;
use crate::setup_custom_style;
use crate::utils::get_current_pak_characteristics;
use eframe::egui;
use eframe::egui::{Align, Checkbox, ComboBox, Context, Label, TextEdit};
use egui_extras::{Column, TableBuilder};
use egui_flex::{item, Flex, FlexAlign};
use log::error;
use repak::utils::AesKey;
use repak::{Compression, Error, PakReader};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::LazyLock;

#[derive(Clone, Debug, Default)]
pub struct InstallableMod {
    pub mod_name: String,
    pub mod_type: String,
    pub repak: bool,
    pub fix_mesh: bool,
    pub is_dir: bool,
    pub editing: bool,
    pub path_hash_seed: String,
    pub mount_point: String,
    pub compression: Compression,
    pub failed_to_install: bool,
    pub reader: Option<PakReader>,
    pub mod_path: PathBuf,
}

#[derive(Debug)]
pub struct ModInstallRequest {
    pub(crate) mods: Vec<InstallableMod>,
    pub mod_directory: PathBuf,

    pub total_mods: f32,
    pub installed_mods: f32,
}
impl ModInstallRequest {
    pub fn new(mods: Vec<InstallableMod>, mod_directory: PathBuf) -> Self {
        let len = mods.len();
        Self {
            mods,
            mod_directory,
            total_mods: len as f32,
            installed_mods: 0.0,
        }
    }
}
impl ModInstallRequest {
    pub fn new_mod_dialog(&mut self, ctx: &egui::Context, show_callback: &mut bool) {
        let viewport_options = egui::ViewportBuilder::default()
            .with_title("Deferred Viewport")
            .with_inner_size([1000.0, 800.0])
            .with_always_on_top();

        Context::show_viewport_immediate(
            &ctx,
            egui::ViewportId::from_hash_of("immediate_viewport"),
            viewport_options,
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                setup_custom_style(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Mods to install");
                    ui.set_min_width(ui.available_width());
                    ui.set_min_height(ui.available_height());
                    // ScrollArea::vertical()
                    //     .auto_shrink([false, false])
                    //     .show(ui, |ui| {
                    self.table_ui(ui);
                    // });
                });
                egui::TopBottomPanel::bottom("bottom_panel")
                    .min_height(50.)
                    .show(ctx, |ui| {
                        // ui.heading("WTF");
                        ui.set_min_width(ui.available_width());
                        ui.set_min_height(ui.available_height());
                        Flex::horizontal()
                            .align_items(FlexAlign::Center)
                            .w_auto()
                            .h_auto()
                            .show(ui, |ui| {
                                let selection_bg_color = ctx.style().visuals.selection.bg_fill;

                                let install_mod = ui.add(
                                    item(),
                                    egui::Button::new("Install mod").fill(selection_bg_color),
                                );

                                let cancel = ui.add(item(), egui::Button::new("Cancel"));
                                cancel.clicked().then(|| {
                                    *show_callback = false;
                                });

                                if install_mod.clicked() {
                                    install_mods_in_viewport(
                                        &mut self.mods,
                                        &self.mod_directory,
                                        &mut self.installed_mods,
                                    );
                                }
                            });
                        ui.add(
                            egui::ProgressBar::new(self.installed_mods / self.total_mods)
                                .text("Installing mods...")
                                .animate(true)
                                .show_percentage(),
                        );
                    });
                if ctx.input(|i| i.viewport().close_requested()) {
                    // Tell parent viewport that we should not show next frame:
                    *show_callback = false;
                }
            },
        );
    }

    fn table_ui(&mut self, ui: &mut egui::Ui) {
        let available_height = ui.available_height();
        ui.separator();

        let table = TableBuilder::new(ui)
            .striped(true)
            // .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
            .column(Column::auto())
            .column(Column::remainder().at_least(400.)) // mod name
            .column(Column::auto()) // Character
            .column(Column::auto()) // type
            .column(Column::remainder()) // options
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        table
            .header(20., |mut header| {
                header.col(|ui| {
                    ui.label("Row");
                });

                header.col(|ui| {
                    ui.label("Name");
                });

                header.col(|ui| {
                    ui.label("Category");
                });

                header.col(|ui| {
                    ui.label("Mod type");
                });
                header.col(|ui| {
                    ui.label("Options");
                });
            })
            .body(|mut body| {
                for (rowidx, mods) in self.mods.iter_mut().enumerate() {
                    body.row(20., |mut row| {
                        row.col(|ui| {
                            ui.add(Label::new(format!("{})", rowidx + 1)).halign(Align::RIGHT));
                        });
                        // name field
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if mods.editing {
                                    ui.set_width(ui.available_width() * 0.8); // padding for edit
                                    let text_edit = ui.add(
                                        egui::TextEdit::singleline(&mut mods.mod_name)
                                            .clip_text(true),
                                    );
                                    if text_edit.lost_focus()
                                        || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                    {
                                        mods.editing = false;
                                    }
                                } else {
                                    ui.add(
                                        Label::new(&mods.mod_name).halign(Align::LEFT).truncate(),
                                    );
                                }
                                // align button right
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button(if mods.editing { "✔" } else { "✏" }).clicked()
                                        {
                                            mods.editing = !mods.editing;
                                        }
                                    },
                                );
                            });
                        });
                        row.col(|ui| {
                            ui.label(&mods.mod_type);
                        });
                        row.col(|ui| {
                            let label = match mods.is_dir {
                                true => "Directory",
                                false => "Pak",
                            };
                            ui.label(label);
                        });
                        row.col(|ui| {
                            ui.collapsing("Options", |ui| {
                                ui.add_enabled(
                                    !mods.is_dir,
                                    Checkbox::new(&mut mods.repak, "To repak"),
                                );
                                ui.add_enabled(
                                    mods.repak,
                                    Checkbox::new(&mut mods.fix_mesh, "Fix mesh"),
                                );
                                let text_edit = TextEdit::singleline(&mut mods.mount_point);
                                ui.add(text_edit.hint_text("Enter mount point..."));

                                // Text edit for path_hash_seed with hint
                                let text_edit = TextEdit::singleline(&mut mods.path_hash_seed);
                                ui.add(text_edit.hint_text("Enter path hash seed..."));

                                ComboBox::new("comp_level", "Compression Algorithm")
                                    .selected_text(format!("{:?}", mods.compression))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut mods.compression,
                                            Compression::Zlib,
                                            "Zlib",
                                        );
                                        ui.selectable_value(
                                            &mut mods.compression,
                                            Compression::Gzip,
                                            "Gzip",
                                        );
                                        ui.selectable_value(
                                            &mut mods.compression,
                                            Compression::Oodle,
                                            "Oodle",
                                        );
                                        ui.selectable_value(
                                            &mut mods.compression,
                                            Compression::Zstd,
                                            "Zstd",
                                        );
                                        ui.selectable_value(
                                            &mut mods.compression,
                                            Compression::LZ4,
                                            "LZ4",
                                        );
                                    });
                            });
                        });
                    })
                }
            });
    }
}

pub const AES_KEY: LazyLock<AesKey> = LazyLock::new(|| {
    AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74")
        .expect("Unable to initialise AES_KEY")
});

pub fn map_paths_to_mods(paths: &Vec<PathBuf>) -> Vec<InstallableMod> {
    paths
        .into_iter()
        .map(|path| {
            let is_dir = path.clone().is_dir();

            let mut modtype = "Unknown".to_string();
            let mut pak = None;

            if !is_dir {
                let builder = repak::PakBuilder::new()
                    .key(AES_KEY.clone().0)
                    .reader(&mut BufReader::new(File::open(path.clone()).unwrap()));

                match builder {
                    Ok(builder) => {
                        pak = Some(builder.clone());
                        modtype = get_current_pak_characteristics(builder.files());
                    }
                    Err(e) => {
                        error!("Error reading pak file: {}", e);
                        return Err(e);
                    }
                }
            }
            if let None = pak {
                assert!(is_dir);
            }

            Ok(InstallableMod {
                mod_name: path.file_stem().unwrap().to_str().unwrap().to_string(),
                mod_type: modtype,
                repak: !is_dir,
                fix_mesh: false,
                is_dir,
                reader: pak,
                mod_path: path.clone(),
                mount_point: "../../../".to_string(),
                path_hash_seed: "00000000".to_string(),
                ..Default::default()
            })
        })
        .filter_map(|x: Result<InstallableMod, repak::Error>| x.ok())
        .collect::<Vec<_>>()
}

pub fn map_dropped_file_to_mods(dropped_files: &Vec<egui::DroppedFile>) -> Vec<InstallableMod> {
    let files = dropped_files
        .into_iter()
        .map(|dropped_file| {
            let is_dir = dropped_file.path.clone().unwrap().is_dir();
            let mut modtype = "Unknown".to_string();

            let pakfile = dropped_file.path.clone().unwrap();
            if !is_dir {
                let mut builder = repak::PakBuilder::new();
                builder = builder.key(AES_KEY.clone().0);

                let pak =
                    builder.reader(&mut BufReader::new(File::open(pakfile.clone()).unwrap()))?;

                modtype = get_current_pak_characteristics(pak.files());
            }

            Ok(InstallableMod {
                mod_name: pakfile.file_stem().unwrap().to_str().unwrap().to_string(),
                mod_type: modtype,
                repak: !is_dir,
                fix_mesh: false,
                mount_point: "../../../".to_string(),
                path_hash_seed: "00000000".to_string(),
                is_dir,
                ..Default::default()
            })
        })
        .filter_map(|x: Result<InstallableMod, repak::Error>| x.ok())
        .collect::<Vec<_>>();
    files
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui::{epaint, Color32, FontData, FontDefinitions, FontFamily, RichText, Slider};
use epaint::{Pos2, Rounding, Vec2};
use forest_egui::{cartesian_to_linear, Cell, CellType};
use reqwest::StatusCode;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(560. * 1.5, 330. * 1.5)),
        ..Default::default()
    };
    eframe::run_native(
        "Forest UI",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

#[derive(Default)]
struct MyApp {
    field: Vec<Cell>,
    field_size: u8,
    c_frame: u8,
    a_frame: u64,
    message: String,
    stats: Stats,
    reqest: Request,
    speed: f32,
    running: bool,
    debug: bool,
}

#[derive(Default, Debug)]
struct Stats {
    flames: usize,
    _empty: usize,
    grass: usize,
    tress: usize,
}

#[derive(Default)]
struct Request {
    size: u8,
    flames: usize,
    grass: usize,
    tress: usize,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            field: vec![],
            field_size: 0,
            c_frame: 0,
            message: "".to_owned(),
            a_frame: 0,
            stats: Stats {
                ..Default::default()
            },
            reqest: Request {
                size: 1,
                ..Default::default()
            },
            running: false,
            debug: false,
            speed: 1.,
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "jb_mono".to_owned(),
        FontData::from_static(include_bytes!(
            "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf"
        )),
    );
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "jb_mono".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "jb_mono".to_owned());

    ctx.set_fonts(fonts);
}

fn get_new_field(req: &Request) -> Result<Vec<Cell>, String> {
    let c = reqwest::blocking::Client::new();
    let req = c
        .get("http://127.0.0.1:3030/v1/field/random")
        .query(&[
            ("size", &req.size.to_string()),
            ("grass", &req.grass.to_string()),
            ("trees", &req.tress.to_string()),
            ("flames", &req.flames.to_string()),
        ])
        .send();

    match req {
        Ok(response) => match response.status() {
            StatusCode::OK => Ok(response.json::<Vec<Cell>>().unwrap()),
            _ => Err(String::from("Server is sad")),
        },
        Err(e) => Err(format!("{}", e)),
    }
}

fn get_field_step(field: &Vec<Cell>) -> Result<Vec<Cell>, String> {
    let c = reqwest::blocking::Client::new();
    let req = c
        .post("http://127.0.0.1:3030/v1/simulation/step")
        .body(serde_json::to_string(&field).unwrap())
        .send();

    match req {
        Ok(response) => match response.status() {
            StatusCode::OK => Ok(response.json::<Vec<Cell>>().unwrap()),
            _ => Err(String::from("Server is sad")),
        },
        Err(e) => Err(format!("{}", e)),
    }
}

fn get_stats(field: &[Cell]) -> Stats {
    Stats {
        _empty: field
            .iter()
            .filter(|c| c.cell_type == CellType::Empty)
            .count(),
        grass: field
            .iter()
            .filter(|c| c.cell_type == CellType::Grass)
            .count(),
        tress: field
            .iter()
            .filter(|c| c.cell_type == CellType::Tree)
            .count(),
        flames: field
            .iter()
            .filter(|c| c.cell_type == CellType::Flame)
            .count(),
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::D)) {
            self.debug = !self.debug;
        }

        ctx.set_pixels_per_point(1.5);
        self.a_frame += 1;
        self.c_frame += 1;
        self.c_frame %= 60;

        if self.c_frame % (60. / self.speed) as u8 == 0 && self.running {
            match get_field_step(&self.field) {
                Ok(field) => {
                    self.field = field;
                    self.stats = get_stats(&self.field);
                }
                Err(e) => self.message = e,
            }
        }

        egui::SidePanel::left("Controls")
            .exact_width(150.)
            .resizable(false)
            .show(ctx, |ui| {
                let pointer = ctx.input(|i| i.pointer.hover_pos());

                ui.horizontal(|ui| {
                    if self.running {
                        if ui.button("stop").clicked() {
                            self.running = false;
                        }
                        ui.spinner();
                    } else if ui.button("start").clicked() {
                        self.running = true;
                    }
                });
                ui.add(
                    Slider::new(&mut self.speed, 1.0..=20.0)
                        .text("speed")
                        .max_decimals(2)
                        .logarithmic(true),
                );
                ui.separator();

                if self.debug {
                    ui.label(RichText::new("debug").size(10.).color(Color32::DARK_GRAY));
                    ui.label(self.c_frame.to_string());
                    ui.label(self.a_frame.to_string());
                    match pointer {
                        Some(v) => {
                            ui.label(format!("{:.1}, {:.1}", v.x, v.y));
                        }
                        None => {
                            ui.label("???, ???");
                        }
                    }
                    ui.label(self.message.clone());

                    if ui.button("update field").clicked() {
                        match get_field_step(&self.field) {
                            Ok(field) => {
                                self.field = field;
                                self.message = "".into();
                                self.stats = get_stats(&self.field);
                            }
                            Err(e) => self.message = e,
                        }
                    }

                    ui.separator();
                }

                ui.label(RichText::new("stats").size(10.).color(Color32::DARK_GRAY));
                ui.label(format!(
                    "grass : {} ({:.1}%)",
                    &self.stats.grass,
                    (self.stats.grass as f32 / self.field.len() as f32) * 100.
                ));
                ui.label(format!(
                    "trees : {} ({:.1}%)",
                    &self.stats.tress,
                    (self.stats.tress as f32 / self.field.len() as f32) * 100.
                ));
                ui.label(format!(
                    "flames: {} ({:.1}%)",
                    &self.stats.flames,
                    (self.stats.flames as f32 / self.field.len() as f32) * 100.
                ));

                ui.separator();

                ui.add(Slider::new(&mut self.reqest.size, 1..=60).text("field size"));
                ui.add(
                    Slider::new(
                        &mut self.reqest.grass,
                        0..=self.reqest.size as usize * self.reqest.size as usize,
                    )
                    .text("grass"),
                );
                ui.add(
                    Slider::new(
                        &mut self.reqest.tress,
                        0..=self.reqest.size as usize * self.reqest.size as usize,
                    )
                    .text("trees"),
                );
                ui.add(
                    Slider::new(
                        &mut self.reqest.flames,
                        0..=self.reqest.size as usize * self.reqest.size as usize,
                    )
                    .text("flames"),
                );

                ui.separator();

                if ui.button("get new field").clicked() {
                    match get_new_field(&self.reqest) {
                        Ok(field) => {
                            self.field = field;
                            self.message = "".into();
                        }
                        Err(e) => self.message = e,
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.field_size = (self.field.len() as f64).sqrt() as u8;
            let mut linear_coord: usize;
            let painter = ui.painter().with_clip_rect(epaint::Rect {
                min: Pos2 { x: 250., y: 0. },
                max: Pos2 { x: 750., y: 600. },
            });
            let field_start = Pos2 { x: 250., y: 15. };
            let sqare_size = 5.;

            for i in 0..self.field_size {
                for j in 0..self.field_size {
                    match cartesian_to_linear(i.into(), j.into(), &self.field) {
                        Ok(v) => linear_coord = v,
                        Err(_) => panic!("AAAAAAAAAAAA"),
                    }

                    let transparency =
                        usize::min(255, self.field[linear_coord].age * (255 / 8)) as u8;

                    painter.rect_filled(
                        epaint::Rect {
                            min: painter.round_pos_to_pixels(
                                field_start
                                    + Vec2 {
                                        x: sqare_size * i as f32,
                                        y: sqare_size * j as f32,
                                    },
                            ),
                            max: painter.round_pos_to_pixels(
                                field_start
                                    + Vec2 {
                                        x: sqare_size * (i + 1) as f32,
                                        y: sqare_size * (j + 1) as f32,
                                    },
                            ),
                        },
                        Rounding::ZERO,
                        match &self.field[linear_coord].cell_type {
                            CellType::Empty => Color32::from_gray(0),
                            CellType::Grass => {
                                Color32::from_rgba_premultiplied(25, 200, 80, transparency)
                            }
                            CellType::Tree => {
                                Color32::from_rgba_premultiplied(80, 150, 80, transparency)
                            }
                            CellType::Flame => {
                                Color32::from_rgba_premultiplied(200, 50, 50, transparency)
                            }
                        },
                    );
                }
            }
        });
    }
}

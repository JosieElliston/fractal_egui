mod fractal;

use eframe::egui;
use fractal::*;

fn main() -> eframe::Result {
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "fractal",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Complex {
    real: f32,
    imag: f32,
}
impl Complex {
    const ZERO: Complex = Complex {
        real: 0.0,
        imag: 0.0,
    };

    // fn zero() -> Self {
    //     Self {
    //         real: 0.0,
    //         imag: 0.0,
    //     }
    // }
}

#[derive(Clone, Copy, Debug)]
// struct Camera {
//     lo: Complex,
//     hi: Complex,
// }
struct Camera {
    center: Complex,
    radius_real: f32,
}
impl Camera {
    // fn from_center_radius(center: Complex, radius: f32) -> Self {
    //     Self {
    //         lo: Complex {
    //             real: center.real - radius,
    //             imag: center.imag - radius,
    //         },
    //         hi: Complex {
    //             real: center.real + radius,
    //             imag: center.imag + radius,
    //         },
    //     }
    // }

    // fn center(&self) -> Complex {
    //     Complex {
    //         real: (self.lo.real + self.hi.real) / 2.0,
    //         imag: (self.lo.imag + self.hi.imag) / 2.0,
    //     }
    // }
}
impl Default for Camera {
    fn default() -> Self {
        // Self {
        //     lo: Complex {
        //         real: -2.0,
        //         imag: -2.0,
        //     },
        //     hi: Complex {
        //         real: 2.0,
        //         imag: 2.0,
        //     },
        // }
        Self {
            center: Complex {
                real: 0.0,
                imag: 0.0,
            },
            radius_real: 2.0,
        }
    }
}
// struct CameraVelocity
// struct CameraMotion

// struct Fractal {
//     fractal: Fractal,
//     needs_update: bool,
// }
// impl Fractal {
//     // fn new(fractal: impl Fractal) -> Self {
//     //     Self {
//     //         fractal: Box::new(fractal),
//     //         needs_update: true,
//     //         camera: Camera::default(),
//     //     }
//     // }

//     fn render(&mut self, ui: &eframe::egui::Ui) {
//         if self.needs_update {
//             self.fractal.render_to_texture();
//             self.needs_update = false;
//         }
//         eframe::egui::widgets::Image::from_texture(eframe::egui::load::SizedTexture::new(
//             self.fractal.texture_id,
//             eframe::egui::Vec2::new(10.0, 10.0), // arbitrary size
//         ))
//         .paint_at(ui, ui.available_rect_before_wrap());
//     }
// }

// struct FractalWindow {
//     fractal: Fractal,
//     is_open: bool,
// }

struct App {
    render_state: eframe::egui_wgpu::RenderState,
    main: Fractal,
    // windows: Vec<FractalWindow>,
    settings_main: bool,
    fractal_windows: Vec<Fractal>,
    settings_windows: Vec<bool>,
    point: Complex,
    show_overlay: bool,
    // /// whether to have nice trackpad panning and zooming at the cost of disabling the mouse
    // trackpad: bool,
    fractal_counter: usize,
    dts: egui::util::History<f32>,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc.wgpu_render_state.clone().unwrap();
        Self {
            main: Fractal::default(&render_state, 0, FractalType::new_mandelbrot(Complex::ZERO)),
            // main: Fractal::default(&render_state, 0, FractalType::new_julia(Complex::ZERO)),
            settings_main: false,
            fractal_windows: vec![],
            settings_windows: vec![],
            point: Complex::ZERO,
            show_overlay: true,
            // trackpad: false,
            render_state,
            fractal_counter: 1,
            dts: egui::util::History::new(2..100, 1.0),
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show(ctx, |ui| {
                self.dts.add(
                    ctx.input(|input_state| input_state.time),
                    ctx.input(|input_state| input_state.stable_dt),
                );

                if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                    self.show_overlay = !self.show_overlay;
                }

                // TODO: possibly fractals should know whether they're main and settings_open
                // TODO: better name for the main fractal so swapping doesn't break the names
                // TODO: clicking on the background should deselect/unfocus the windows
                {
                    let FractalUiResponse {
                        should_open_settings: open,
                        new_point: point,
                    } = self.main.ui(
                        ctx,
                        ui,
                        match self.show_overlay {
                            true => Some(self.point),
                            false => None,
                        },
                    );
                    self.settings_main |= open;
                    if let Some(point) = point {
                        self.point = point;
                    }
                }
                if self.show_overlay {
                    if self.settings_main {
                        let SettingsUiResponse {
                            is_settings_open,
                            swap_main: _,
                        } = self.main.settings_ui(ctx, ui);
                        self.settings_main = is_settings_open;
                    }
                    assert_eq!(self.fractal_windows.len(), self.settings_windows.len());
                    let mut i = 0;
                    while i < self.fractal_windows.len() {
                        // TODO: better title name
                        // TODO: make title smaller
                        // TODO: make it not have a shadow
                        let fractal = &mut self.fractal_windows[i];
                        let mut fractal_open = true;
                        egui::Window::new(fractal.name())
                            .resizable(true)
                            // .shadow(egui::Shadow::NONE)
                            // .title_bar(false)
                            // .default_open(default_open)
                            .default_size([250.0, 250.0])
                            .open(&mut fractal_open)
                            .show(ctx, |ui| {
                                let FractalUiResponse {
                                    should_open_settings,
                                    new_point,
                                } = fractal.ui(ctx, ui, Some(self.point));
                                self.settings_windows[i] |= should_open_settings;
                                if let Some(point) = new_point {
                                    self.point = point;
                                }
                            });
                        if self.settings_windows[i] {
                            let SettingsUiResponse {
                                is_settings_open,
                                swap_main,
                            } = fractal.settings_ui(ctx, ui);
                            self.settings_windows[i] = is_settings_open;
                            if swap_main {
                                std::mem::swap(&mut self.main, &mut self.fractal_windows[i]);
                                std::mem::swap(
                                    &mut self.settings_main,
                                    &mut self.settings_windows[i],
                                );
                            }
                        }
                        if !fractal_open {
                            self.fractal_windows.swap_remove(i);
                            self.settings_windows.swap_remove(i);
                        } else {
                            i += 1;
                        }
                    }

                    // area is to allow the frame to be drawn on top of the fractal
                    egui::Area::new(egui::Id::new("area"))
                        .constrain_to(ctx.screen_rect())
                        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .outer_margin(5.0)
                                .shadow(egui::Shadow::NONE)
                                .show(ui, |ui| {
                                    egui::CollapsingHeader::new("global").show(ui, |ui| {
                                        let average_dt = self.dts.average().expect("we added one this frame so dts must be non-empty");
                                        ui.label(format!("    dt: {:08.05}\n1/dt: {:08.05}", average_dt, 1.0 / average_dt,));
                                        // println!(
                                        //     "center: {} + {}i, real_radius: {}",
                                        //     self.main.camera().center.real,
                                        //     self.main.camera().center.imag,
                                        //     self.main.camera().radius_real,
                                        // );
                                        // TODO: clicking copies the camera?
                                        ui.label(format!(
                                            "center: {:12.09} + {:12.09}i\nreal_radius: {:12.09}",
                                            self.main.camera().center.real,
                                            self.main.camera().center.imag,
                                            self.main.camera().radius_real,
                                        ));

                                        ui.add(
                                            egui::Slider::new(&mut self.point.real, -2.0..=2.0)
                                                .text("point real"),
                                        );
                                        ui.add(
                                            egui::Slider::new(&mut self.point.imag, -2.0..=2.0)
                                                .text("point imag"),
                                        );

                                        if ui.button("add mandelbrot").clicked() {
                                            self.fractal_windows.push(Fractal::default(
                                                &self.render_state,
                                                self.fractal_counter,
                                                FractalType::new_mandelbrot(Complex::ZERO),
                                            ));
                                            self.settings_windows.push(false);
                                            self.fractal_counter += 1;
                                        }
                                        if ui.button("add julia set").clicked() {
                                            self.fractal_windows.push(Fractal::default(
                                                &self.render_state,
                                                self.fractal_counter,
                                                FractalType::new_julia(Complex::ZERO),
                                            ));
                                            self.settings_windows.push(false);
                                            self.fractal_counter += 1;
                                        }
                                    });
                                });
                        });
                }

                // // TODO: point may not the the correct abstraction
                // if self.point != prev_point {
                //     self.main.set_point(self.point);
                //     for fractal in &mut self.fractal_windows {
                //         fractal.set_point(self.point);
                //     }
                // }
            });
    }
}

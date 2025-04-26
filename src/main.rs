mod fractals;

use eframe::egui::{self, Vec2};
use fractals::fractal::*;

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

#[derive(Clone, Copy, Debug)]
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
    main: Fractal,
    // windows: Vec<FractalWindow>,
    settings_main: bool,
    fractal_windows: Vec<(Fractal, String)>,
    settings_windows: Vec<bool>,
    show_overlay: bool,
    /// whether to have nice trackpad panning and zooming at the cost of disabling the mouse
    trackpad: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            main: Fractal::default(cc, FractalType::default()),
            settings_main: false,
            // fractal_windows: vec![],
            fractal_windows: vec![(
                Fractal::default(cc, FractalType::default()),
                "test".to_string(),
            )],
            settings_windows: vec![false],
            show_overlay: true,
            trackpad: false,
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show(ctx, |ui| {
                let dt = ctx.input(|input_state| input_state.stable_dt);
                // println!("dt: {:?}", dt);
                // let _ = ui.button(format!("dt: {:?}", dt));
                // let _ = ui.button(format!("1/dt: {:?}", 1.0 / dt));

                if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                    self.show_overlay = !self.show_overlay;
                }

                // TODO: remove name
                // TODO: clicking on the background should deselect/unfocus the windows
                self.settings_main |= self.main.ui(ctx, ui);
                if self.show_overlay {
                    if self.settings_main {
                        self.settings_main = self.main.settings_ui(ctx, ui, "main");
                    }
                    assert_eq!(self.fractal_windows.len(), self.settings_windows.len());
                    let mut i = 0;
                    while i < self.fractal_windows.len() {
                        // TODO: better title name
                        // TODO: make title smaller
                        // TODO: make it not have a shadow
                        let (fractal, name) = &mut self.fractal_windows[i];
                        let mut fractal_open = true;
                        egui::Window::new("fractal")
                            .resizable(true)
                            // .title_bar(false)
                            // .default_open(default_open)
                            .default_size([250.0, 250.0])
                            .open(&mut fractal_open)
                            .show(ctx, |ui| {
                                self.settings_windows[i] |= fractal.ui(ctx, ui);
                            });
                        if self.settings_windows[i] {
                            self.settings_windows[i] = fractal.settings_ui(ctx, ui, &name);
                        }
                        if !fractal_open {
                            self.fractal_windows.swap_remove(i);
                            self.settings_windows.swap_remove(i);
                        } else {
                            i += 1;
                        }
                    }

                    // settings ui
                    // egui::Frame::popup(ui.style())
                    //     // .outer_margin(10.0)
                    //     // .shadow(egui::Shadow::NONE)
                    //     // .stroke(egui::Stroke::NONE)
                    //     .show(ui, |ui| {
                    //         egui::CollapsingHeader::new("settings").show(ui, |ui| {});
                    //     });
                }
            });
    }
}

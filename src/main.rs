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
    fractal_windows: Vec<(Fractal, String)>,
    show_overlay: bool,
    /// whether to have nice trackpad panning and zooming at the cost of disabling the mouse
    trackpad: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            main: Fractal::default(cc, FractalType::default()),
            // fractal_windows: vec![],
            fractal_windows: vec![(Fractal::default(cc, FractalType::default()), "test".into())],
            // settings_windows: vec![],
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

                self.main.render_to_ui(ctx, ui, "main");
                if self.show_overlay {
                    self.fractal_windows.retain_mut(|(fractal, name)| {
                        // TODO: better title name
                        // TODO: make title smaller
                        // TODO: make it not have a shadow
                        let mut open = true;
                        egui::Window::new(&*name)
                            // .open(&mut is_open)
                            // .vscroll(false)
                            .resizable(true)
                            // .title_bar(false)
                            // .default_open(default_open)
                            .default_size([250.0, 250.0])
                            .open(&mut open)
                            .show(ctx, |ui| {
                                // Scene::new()
                                //     .max_inner_size([350.0, 1000.0])
                                //     .zoom_range(0.1..=2.0)
                                //     .show(ui, &mut self.scene_rect, |ui| {
                                //         reset_view = ui.button("Reset view").clicked();

                                //         ui.add_space(16.0);

                                //         self.widget_gallery.ui(ui);

                                //         ui.put(
                                //             Rect::from_min_size(
                                //                 Pos2::new(0.0, -64.0),
                                //                 Vec2::new(200.0, 16.0),
                                //             ),
                                //             egui::Label::new("You can put a widget anywhere")
                                //                 .selectable(false),
                                //         );

                                //         inner_rect = ui.min_rect();
                                //     })
                                //     .response;

                                fractal.render_to_ui(ctx, ui, name);
                                ui.allocate_space(ui.available_size());
                            });
                        open
                    });

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

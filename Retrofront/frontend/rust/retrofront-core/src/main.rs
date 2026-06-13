use eframe::egui;
use retrofront_core::{MenuDriver, UiRuntime};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Retrofront - RetroArch UI Mock")
            .with_inner_size([1280.0, 720.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Retrofront",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_pixels_per_point(1.0);
            Box::new(RetrofrontApp {
                rt: UiRuntime::default(),
            })
        }),
    )
}
struct RetrofrontApp {
    rt: UiRuntime,
}
impl eframe::App for RetrofrontApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let rect = ctx.screen_rect();
        self.rt.begin_frame(
            rect.width() as u32,
            rect.height() as u32,
            ctx.pixels_per_point(),
        );
        handle_input(ctx, &mut self.rt);
        draw(ctx, &mut self.rt);
        self.rt.end_frame();
        ctx.request_repaint();
    }
}
fn handle_input(ctx: &egui::Context, rt: &mut UiRuntime) {
    ctx.input(|i| {
        if i.key_pressed(egui::Key::ArrowDown) {
            rt.move_sel(1)
        }
        if i.key_pressed(egui::Key::ArrowUp) {
            rt.move_sel(-1)
        }
        if i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space) {
            rt.activate()
        }
        if i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Backspace) {
            rt.back()
        }
        if i.key_pressed(egui::Key::ArrowLeft) {
            rt.change_value(-1)
        }
        if i.key_pressed(egui::Key::ArrowRight) {
            rt.change_value(1)
        }
    });
}
fn draw(ctx: &egui::Context, rt: &mut UiRuntime) {
    let bg = match rt.driver {
        MenuDriver::Xmb => egui::Color32::from_rgb(18, 36, 80),
        MenuDriver::Ozone => egui::Color32::from_rgb(28, 30, 34),
        MenuDriver::Rgui => egui::Color32::from_rgb(0, 40, 46),
        MenuDriver::MaterialUi => egui::Color32::from_rgb(32, 40, 56),
    };
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(bg))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for d in MenuDriver::all() {
                    let selected = d == rt.driver;
                    if ui.selectable_label(selected, d.label()).clicked() {
                        rt.set_driver(d)
                    }
                }
            });
            ui.separator();
            match rt.driver {
                MenuDriver::Xmb => xmb(ui, rt),
                MenuDriver::Ozone => ozone(ui, rt),
                MenuDriver::Rgui => rgui(ui, rt),
                MenuDriver::MaterialUi => material(ui, rt),
            };
            notifications(ui, rt);
        });
}
fn list(ui: &mut egui::Ui, rt: &mut UiRuntime, large: bool) {
    let items = rt.items();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (idx, it) in items.iter().enumerate() {
            let sel = idx == rt.selected;
            let label = format!("{}   {}", it.label, it.value);
            let text = if it.enabled {
                egui::RichText::new(label)
            } else {
                egui::RichText::new(label).color(egui::Color32::GRAY)
            };
            let resp = ui.selectable_label(sel, text);
            if resp.hovered() {
                rt.selected = idx;
            }
            if resp.clicked() {
                rt.activate();
            }
            if large {
                ui.label(
                    egui::RichText::new(&it.sublabel)
                        .small()
                        .color(egui::Color32::LIGHT_GRAY),
                );
            }
        }
    });
}
fn xmb(ui: &mut egui::Ui, rt: &mut UiRuntime) {
    ui.heading(format!("✦ XMB / {}", rt.current_screen()));
    ui.horizontal(|ui|{ui.vertical(|ui|list(ui,rt,true)); ui.group(|ui|{ui.heading("Thumbnail"); ui.label("▧ placeholder"); ui.label("Wallpaper, alpha animation, icons and breadcrumbs are represented by generated UI assets.");});});
}
fn ozone(ui: &mut egui::Ui, rt: &mut UiRuntime) {
    ui.heading(format!("☰ Ozone / {}", rt.current_screen()));
    ui.columns(2, |cols| {
        cols[0].vertical(|ui| {
            ui.label("Sidebar");
            for s in ["Main Menu", "Settings", "History", "Information"] {
                if ui.button(s).clicked() {
                    rt.stack = vec![s.into()];
                    rt.selected = 0;
                }
            }
        });
        cols[1].vertical(|ui| list(ui, rt, true));
    });
}
fn rgui(ui: &mut egui::Ui, rt: &mut UiRuntime) {
    ui.monospace(format!(
        "RGUI > {}  {}x{}",
        rt.current_screen(),
        rt.width,
        rt.height
    ));
    ui.separator();
    list(ui, rt, false);
}
fn material(ui: &mut egui::Ui, rt: &mut UiRuntime) {
    ui.heading(format!("● Material UI / {}", rt.current_screen()));
    ui.add_space(8.0);
    list(ui, rt, true);
}
fn notifications(ui: &mut egui::Ui, rt: &UiRuntime) {
    egui::TopBottomPanel::bottom("notes").show_inside(ui, |ui| {
        for n in &rt.notifications {
            ui.label(format!("ⓘ {n}"));
        }
    });
}

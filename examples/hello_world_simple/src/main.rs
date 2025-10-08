#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{
    self, Button, Id, ScrollArea, Sense,
    style_trait::{HasClasses as _, Modifiers},
    vec2,
};
use hello_world_simple::css_engine::CssEngine;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    // Our application state:
    let mut name = "Arthur".to_owned();
    let mut age = 42;
    let mut engine = CssEngine::default();

    // engine.add_properties(&Classes::from(["test"]), modifier, properties);

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        // ctx.set_style_engine();
        // ctx.style_mut(|s| {
        //     s.visuals.widgets.active.bg_fill = Color32::RED;
        // });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                age += 1;
            }
            ui.label(format!("Hello '{name}', age {age}"));

            ui.add(Button::new("Test").with_class("test"));

            let id = ui.id();
            let test_id = Id::new("test");
            println!("testid {test_id:?}");
            ScrollArea::both()
                .id_salt("test")
                .show_viewport(ui, |ui, vrect| {
                    ui.set_min_size(vec2(1000.0, 1000.0));

                    let area_id = id.with(test_id).with("area");
                    let content_response_option = ui.interact(vrect, area_id, Sense::drag());

                    // let x = content_response_option.dragged();
                    // println!(
                    //     "UH {:?} {:?} {:?} {:?}",
                    //     x,
                    //     ui.clip_rect(),
                    //     id.with(test_id),
                    //     area_id
                    // );
                })
        });
    })
}

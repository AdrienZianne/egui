#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use std::sync::Arc;

use eframe::egui::{
    self, Button, Color32, ComboBox, Frame, Margin, Panel, UiBuilder,
    mutex::Mutex,
    widget_style::{ButtonStyle, HasClasses as _, WidgetStyle},
};

use crate::custom_engine::{CustomThemePluginA, CustomThemePluginB};

mod custom_engine;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    let mut style_code = String::new();
    let mut toggled = false;
    let mut selected = None;
    let ctpa = Arc::new(Mutex::new(CustomThemePluginA::default()));

    let ctpb = Arc::new(Mutex::new(CustomThemePluginB));

    eframe::run_ui_native("My egui App", options, move |ui, _frame| {
        // Register the theme plugin and which style they implement
        ui.add_theme::<WidgetStyle>(&ctpa);
        ui.add_theme::<ButtonStyle>(&ctpa);

        ui.add_theme::<WidgetStyle>(&ctpb);
        ui.add_theme::<ButtonStyle>(&ctpb);

        ui.scope_builder(UiBuilder::new().with_class("body"), |ui| {
            ui.label("body");
            ui.label("central panel");

            Panel::left("style_code").show_inside(ui, |ui| {
                ui.scope_builder(UiBuilder::new().with_class("panel_left"), |ui| {
                    ui.label(
                        "Live editor\n(type color hex to change the color of the dynamic button)",
                    );

                    if ui.text_edit_multiline(&mut style_code).changed() {
                        if let Ok(color) = Color32::from_hex(&style_code) {
                            ctpa.lock().color = Some(color);
                        } else {
                            ctpa.lock().color = None;
                        }
                        ui.invalidate_cache();
                    }
                });
            });

            ui.scope_builder(UiBuilder::new().with_class("grid"), |ui| {
                Frame::new().inner_margin(Margin::same(10)).show(ui, |ui| {
                    ui.scope_builder(UiBuilder::new().with_class("frame1"), |ui| {
                        let mut parent = Some(ui.stack());
                        let mut text = vec![];
                        let mut i: i32 = 0;
                        while let Some(p) = parent {
                            text.push(format!(
                                "{}{}class : '{}', kind : {:?}",
                                " ".repeat((2 * 0_i32.max(i - 1) + 1.min(i)) as usize),
                                if i > 0 { "\\- " } else { "" },
                                p.classes,
                                p.kind()
                            ));
                            i += 1;
                            parent = p.parent.as_ref();
                        }
                        ui.label(format!(
                            "Current hierarchy (child to root):\n{}",
                            text.join("\n")
                        ));
                    })
                })
            });

            ui.add(Button::new("Normal"));
            ui.add(Button::new("red").with_class("red"));
            ui.add(Button::new("blue").with_class("blue"));
            ui.add(Button::new("dynamic in engine A").with_class("dynamic"));
            if ui
                .add(
                    Button::new("red/blue")
                        .with_class_if("red", toggled)
                        .with_class_if("blue", !toggled),
                )
                .clicked()
            {
                toggled = !toggled;
            }

            let before = selected.clone();
            ComboBox::from_label("The current engine")
                .selected_text(format!(
                    "{:?}",
                    ui.current_theme().unwrap_or_else(|| "None".to_owned())
                ))
                .show_ui(ui, |ui| {
                    for i in ui.availables_themes() {
                        ui.selectable_value(&mut selected, Some(i.clone()), format!("{i:?}"));
                    }
                });
            if let Some(selected) = selected.clone()
                && before.is_none_or(|b| b != selected)
            {
                ui.switch_theme(&selected);
            }
        });
    })
}

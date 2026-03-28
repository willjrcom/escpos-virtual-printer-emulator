use crate::emulator::EmulatorState;
use egui::{ScrollArea, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ReceiptViewer {
    show_paper_edges: bool,
    show_grid: bool,
}

impl Default for ReceiptViewer {
    fn default() -> Self {
        Self {
            show_paper_edges: true,
            show_grid: false,
        }
    }
}

impl ReceiptViewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut Ui, emulator_state: &Arc<Mutex<EmulatorState>>) {
        ui.heading("🖨️ Receipt Viewer");
        ui.separator();

        // Controls
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_paper_edges, "Show paper edges");
            ui.checkbox(&mut self.show_grid, "Show grid");
            
            if ui.button("🗑️ Clear").clicked() {
                if let Ok(mut state) = emulator_state.try_lock() {
                    state.clear_printer_buffer();
                }
            }
        });

        ui.separator();

        // Receipt display area
        ScrollArea::both().show(ui, |ui| {
            if let Ok(state) = emulator_state.try_lock() {
                self.render_receipt(ui, &state);
            } else {
                ui.label("Cannot load emulator state");
            }
        });
    }

    fn render_receipt(&self, ui: &mut Ui, state: &EmulatorState) {
        let printer_state = state.get_printer_state();
        let buffer = printer_state.get_buffer();
        
        if buffer.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No receipt data available");
                ui.label("Send ESC/POS commands to see the receipt here");
            });
            return;
        }

        // Paper simulation
        ui.vertical_centered(|ui| {
            let paper_width = 300.0; // Proporcional a 80mm para exibição
            
            egui::Frame::none()
                .fill(egui::Color32::WHITE)
                .outer_margin(10.0)
                .inner_margin(egui::Margin::symmetric(20.0, 10.0))
                .shadow(egui::epaint::Shadow {
                    extrusion: 8.0,
                    color: egui::Color32::from_black_alpha(20),
                })
                .show(ui, |ui| {
                    ui.set_width(paper_width);
                    ui.spacing_mut().item_spacing.y = 2.0;
                    
                    for line in buffer {
                        let alignment = match line.justification {
                            crate::escpos::commands::Justification::Left => egui::Align::Min,
                            crate::escpos::commands::Justification::Center => egui::Align::Center,
                            crate::escpos::commands::Justification::Right => egui::Align::Max,
                        };
                        
                        ui.with_layout(egui::Layout::top_down(alignment), |ui| {
                            if line.text.is_empty() {
                                ui.label(" ");
                            } else {
                                let mut text = egui::RichText::new(&line.text)
                                    .color(egui::Color32::BLACK)
                                    .family(egui::FontFamily::Monospace)
                                    .size(line.font_size as f32 + 2.0);
                                
                                if line.emphasis {
                                    text = text.strong();
                                }
                                if line.italic {
                                    text = text.italics();
                                }
                                if line.underline {
                                    text = text.underline();
                                }
                                
                                ui.label(text);
                            }
                        });
                    }
                });
            
            ui.add_space(20.0);
            ui.label("✂️ Cut line");
        });
    }
}

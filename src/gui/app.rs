use crate::emulator::EmulatorState;
use crate::gui::{CommandLog, ReceiptViewer, SettingsPanel};
use eframe::egui::{CentralPanel, TopBottomPanel};

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Receipt,
    Commands,
    Settings,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Receipt
    }
}

pub struct EscPosEmulatorApp {
    pub emulator_state: std::sync::Arc<tokio::sync::Mutex<EmulatorState>>,
    selected_tab: Tab,
    receipt_viewer: ReceiptViewer,
    command_log: CommandLog,
    settings_panel: SettingsPanel,
}

impl Default for EscPosEmulatorApp {
    fn default() -> Self {
        Self {
            emulator_state: std::sync::Arc::new(tokio::sync::Mutex::new(EmulatorState::new())),
            selected_tab: Tab::Receipt,
            receipt_viewer: ReceiptViewer::new(),
            command_log: CommandLog::new(),
            settings_panel: SettingsPanel::default(),
        }
    }
}

impl EscPosEmulatorApp {
    pub fn new(emulator_state: std::sync::Arc<tokio::sync::Mutex<EmulatorState>>, cc: &eframe::CreationContext<'_>) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        Self {
            emulator_state,
            ..Default::default()
        }
    }

    fn configure_fonts(ctx: &eframe::egui::Context) {
        let fonts = eframe::egui::FontDefinitions::default();
        
        // Ensure we use a font that supports Latin-1 characters
        // On most systems, egui's default font (Ubuntu) should work for ½, ¼, etc.
        // if they are properly encoded. But we can explicitly add ranges if needed.
        
        ctx.set_fonts(fonts);
    }
}

impl eframe::App for EscPosEmulatorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
}

impl EscPosEmulatorApp {
    fn show(&mut self, ctx: &eframe::egui::Context) {
        // Top panel with tabs
        TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Receipt, "🖨️ Receipt");
                ui.selectable_value(&mut self.selected_tab, Tab::Commands, "📋 Commands");
                ui.selectable_value(&mut self.selected_tab, Tab::Settings, "⚙️ Settings");
            });
        });

        // Central panel with content
        CentralPanel::default().show(ctx, |ui| {
            match self.selected_tab {
                Tab::Receipt => {
                    self.receipt_viewer.show(ui, &self.emulator_state);
                }
                Tab::Commands => {
                    self.command_log.show(ui, &self.emulator_state);
                }
                Tab::Settings => {
                    if let Ok(mut state) = self.emulator_state.try_lock() {
                        self.settings_panel.show(ui, &mut state);
                    }
                }
            }
        });
    }
}

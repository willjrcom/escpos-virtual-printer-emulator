use crate::escpos::commands::{EscPosCommand, Font, Justification};
use image::{ImageBuffer, Rgb, RgbImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaperWidth {
    Width50mm,  // 384 dots (48 chars normal font)
    Width78mm,  // 576 dots (72 chars normal font)
    Width80mm,  // 640 dots (80 chars normal font)
}

impl PaperWidth {
    pub fn get_width_dots(&self) -> u32 {
        match self {
            PaperWidth::Width50mm => 384,
            PaperWidth::Width78mm => 576,
            PaperWidth::Width80mm => 640,
        }
    }
    
    pub fn get_max_chars(&self, font_size: u32) -> u32 {
        let dots = self.get_width_dots();
        // Calculer le nombre max de caractères selon la taille de police
        match font_size {
            8..=12 => dots / 8,   // Police normale
            13..=16 => dots / 10, // Police moyenne
            17..=24 => dots / 12, // Police grande
            _ => dots / 8,        // Par défaut
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintLine {
    pub text: String,
    pub justification: Justification,
    pub font: Font,
    pub emphasis: bool,
    pub underline: bool,
    pub italic: bool,
    pub font_size: u32,
}

impl PrintLine {
    pub fn new(justification: Justification, font: Font, emphasis: bool, underline: bool, italic: bool, font_size: u32) -> Self {
        Self {
            text: String::new(),
            justification,
            font,
            emphasis,
            underline,
            italic,
            font_size,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterState {
    pub paper_width: PaperWidth,
    pub current_font: Font,
    pub justification: Justification,
    pub emphasis: bool,
    pub underline: bool,
    pub italic: bool,
    pub buffer: Vec<PrintLine>,
    pub line_height: u32,
    pub font_size: u32,
    pub dpi: u32,
}

impl PrinterState {
    pub fn new() -> Self {
        Self {
            paper_width: PaperWidth::Width80mm,
            current_font: Font::FontA,
            justification: Justification::Left,
            emphasis: false,
            underline: false,
            italic: false,
            buffer: Vec::new(),
            line_height: 24,
            font_size: 12,
            dpi: 180,
        }
    }

    pub fn process_command(&mut self, command: &EscPosCommand) {
        match command {
            EscPosCommand::Text(text) => {
                self.add_text(text);
            }
            EscPosCommand::NewLine => {
                self.add_new_line();
            }
            EscPosCommand::SetFont(font) => {
                self.current_font = font.clone();
            }
            EscPosCommand::SetJustification(justification) => {
                self.justification = justification.clone();
            }
            EscPosCommand::SetEmphasis(enabled) => {
                self.emphasis = *enabled;
            }
            EscPosCommand::SetUnderline(enabled) => {
                self.underline = *enabled;
            }
            EscPosCommand::SetItalic(enabled) => {
                self.italic = *enabled;
            }
            EscPosCommand::CutPaper => {
                // Simuler la coupe du papier
                self.add_separator();
            }
            EscPosCommand::PrintImage(_image_data) => {
                // TODO: Implémenter l'affichage d'image
                self.add_text("[IMAGE]");
            }
            EscPosCommand::SetLineHeight(height) => {
                self.line_height = *height;
            }
            EscPosCommand::SetFontSize(size) => {
                self.font_size = *size;
            }
            EscPosCommand::SetCodePage(_code_page) => {
                // The parser handles decoding, so we don't need to do anything here
                // but we could store it if we wanted to show it in the UI
            }
            EscPosCommand::Unknown(_) => {
                // Ignorer les commandes inconnues
            }
            _ => {
                // Ignorer les autres commandes
            }
        }
    }

    fn add_text(&mut self, text: &str) {
        if self.buffer.is_empty() {
            self.add_new_line();
        }

        if let Some(last_line) = self.buffer.last_mut() {
            // Vérifier si le texte dépasse la largeur du papel
            let max_chars = self.paper_width.get_max_chars(last_line.font_size);
            let current_length = last_line.text.chars().count();
            
            if current_length + text.chars().count() > max_chars as usize {
                // Le texte dépasse, criar uma nova linha
                self.add_new_line();
                self.buffer.last_mut().unwrap().text.push_str(text);
            } else {
                last_line.text.push_str(text);
            }
        }
    }

    fn add_new_line(&mut self) {
        self.buffer.push(PrintLine::new(
            self.justification.clone(),
            self.current_font.clone(),
            self.emphasis,
            self.underline,
            self.italic,
            self.font_size,
        ));
    }

    fn add_separator(&mut self) {
        let max_chars = self.paper_width.get_max_chars(self.font_size);
        let separator = "-".repeat(max_chars as usize);
        self.buffer.push(PrintLine {
            text: separator,
            justification: Justification::Center, // Separadores geralmente são centralizados
            font: Font::FontA,
            emphasis: false,
            underline: false,
            italic: false,
            font_size: self.font_size,
        });
        self.add_new_line();
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    pub fn get_buffer(&self) -> &[PrintLine] {
        &self.buffer
    }

    pub fn get_paper_width_dots(&self) -> u32 {
        self.paper_width.get_width_dots()
    }

    pub fn get_printing_width_dots(&self) -> u32 {
        // Largeur d'impression = largeur du papier - marges
        let dots = self.paper_width.get_width_dots();
        dots.saturating_sub(30) // 8mm = ~30 dots de marges
    }

    pub fn render_receipt(&self) -> RgbImage {
        let width = self.get_paper_width_dots() as u32;
        let height = self.calculate_total_height();
        
        let mut image = ImageBuffer::new(width, height);
        
        // Remplir avec du blanc
        for pixel in image.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }
        
        // Rendu du texte (simplifié)
        let mut y_offset = 0;
        for line in &self.buffer {
            if !line.is_empty() {
                self.render_text_line(&mut image, line, y_offset);
            }
            y_offset += self.line_height;
        }
        
        image
    }

    fn render_text_line(&self, _image: &mut RgbImage, _text: &str, _y_offset: u32) {
        // TODO: Implémenter le rendu du texte
        // Pour l'instant, c'est juste un placeholder
    }

    pub fn calculate_total_height(&self) -> u32 {
        self.buffer.len() as u32 * self.line_height
    }

    // Nouvelles méthodes pour les paramètres
    pub fn set_paper_width(&mut self, width: PaperWidth) {
        self.paper_width = width;
    }

    pub fn set_line_height(&mut self, height: u32) {
        self.line_height = height;
    }

    pub fn set_font_size(&mut self, size: u32) {
        self.font_size = size;
    }
}

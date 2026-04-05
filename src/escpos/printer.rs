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
    
    pub fn get_max_chars(&self, _font_size: u32) -> u32 {
        // Valores padrão para impressoras térmicas ESC/POS
        match self {
            PaperWidth::Width50mm => 32, // Comum em impressoras de 58mm
            PaperWidth::Width78mm => 42, // Comum em impressoras de 80mm (Font A)
            PaperWidth::Width80mm => 48, // Padrão para 80mm (Font A - 12x24)
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
            EscPosCommand::InitializePrinter => {
                let saved_buffer = self.buffer.clone();
                let paper_width = self.paper_width.clone();
                *self = Self::new();
                self.buffer = saved_buffer;
                self.paper_width = paper_width;
                // Force a new line with the reset state
                self.add_new_line();
            }
            EscPosCommand::LineFeed => {
                self.add_new_line();
            }
            EscPosCommand::CarriageReturn => {
                // Pour cet émulateur, CR est traité comme un début de ligne si vide
                // ou ignoré s'il y a déjà du texte (simplification)
            }
            EscPosCommand::Unknown(_) => {
                // Ignorer les commandes inconnues
            }
        }
    }

    fn add_text(&mut self, text: &str) {
        if self.buffer.is_empty() {
            self.add_new_line();
        }

        if let Some(last_line) = self.buffer.last_mut() {
            // Se a linha estiver vazia, adota as configurações atuais da impressora
            // Isso garante que comandos de alinhamento/estilo enviados após o início da linha funcionem
            if last_line.text.is_empty() {
                last_line.justification = self.justification.clone();
                last_line.font = self.current_font.clone();
                last_line.emphasis = self.emphasis;
                last_line.underline = self.underline;
                last_line.italic = self.italic;
                last_line.font_size = self.font_size;
            }

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
        // Simuler un espace de coupe au lieu d'ajouter des tirets (qui doublent souvent le footer)
        self.add_new_line();
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
            if !line.text.is_empty() {
                self.render_text_line(&mut image, &line.text, y_offset);
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

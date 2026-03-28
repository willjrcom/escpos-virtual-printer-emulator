use crate::escpos::commands::{EscPosCommand, Font, Justification};
use anyhow::Result;

pub struct EscPosParser {
    buffer: Vec<u8>,
    current_code_page: u8, // ESC t n
}

impl EscPosParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            current_code_page: 0, // Default PC437
        }
    }

    pub fn parse_stream(&mut self, data: &[u8]) -> Result<Vec<EscPosCommand>> {
        self.buffer.extend_from_slice(data);
        let mut commands = Vec::new();
        let mut i = 0;

        while i < self.buffer.len() {
            match self.buffer[i] {
                // Commandes de base
                b'\n' => {
                    commands.push(EscPosCommand::NewLine);
                    i += 1;
                }
                b'\r' => {
                    commands.push(EscPosCommand::CarriageReturn);
                    i += 1;
                }
                b'\x1B' => {
                    // Séquence ESC
                    if i + 1 < self.buffer.len() {
                        let (command, consumed) = self.parse_esc_command(&self.buffer[i..])?;
                        if let Some(cmd) = command {
                            if let EscPosCommand::SetCodePage(cp) = cmd {
                                self.current_code_page = cp;
                            }
                            commands.push(cmd);
                            i += consumed;
                        } else {
                            break; // Attendre plus de données
                        }
                    } else {
                        break; // Attendre plus de données
                    }
                }
                b'\x1D' => {
                    // Séquence GS
                    if i + 1 < self.buffer.len() {
                        let (command, consumed) = self.parse_gs_command(&self.buffer[i..])?;
                        if let Some(cmd) = command {
                            commands.push(cmd);
                            i += consumed;
                        } else {
                            break; // Attendre plus de données
                        }
                    } else {
                        break; // Attendre plus de données
                    }
                }
                _ => {
                    // Texte normal
                    let text_start = i;
                    while i < self.buffer.len() && self.buffer[i] != b'\x1B' && self.buffer[i] != b'\x1D' && self.buffer[i] != b'\n' && self.buffer[i] != b'\r' {
                        i += 1;
                    }
                    if i > text_start {
                        let bytes = &self.buffer[text_start..i];
                        let text = match self.current_code_page {
                            16 => {
                                // Latin-1
                                let (res, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
                                res.to_string()
                            }
                            _ => String::from_utf8_lossy(bytes).to_string(),
                        };
                        
                        if !text.is_empty() {
                            commands.push(EscPosCommand::Text(text));
                        }
                    }
                }
            }
        }

        // Nettoyer le buffer des données traitées
        if i > 0 {
            self.buffer.drain(0..i);
        }

        Ok(commands)
    }

    fn parse_esc_command(&self, data: &[u8]) -> Result<(Option<EscPosCommand>, usize)> {
        if data.len() < 2 {
            return Ok((None, 0));
        }

        match data[1] {
            // Initialisation de l'imprimante
            b'@' => Ok((Some(EscPosCommand::InitializePrinter), 2)),

            // Sélection de la police
            b'M' => {
                if data.len() >= 3 {
                    let font = match data[2] {
                        0 => Font::FontA,
                        1 => Font::FontB,
                        2 => Font::FontC,
                        _ => Font::FontA,
                    };
                    Ok((Some(EscPosCommand::SetFont(font)), 3))
                } else {
                    Ok((None, 0))
                }
            }

            // Justification
            b'a' => {
                if data.len() >= 3 {
                    let justification = match data[2] {
                        0 => Justification::Left,
                        1 => Justification::Center,
                        2 => Justification::Right,
                        _ => Justification::Left,
                    };
                    Ok((Some(EscPosCommand::SetJustification(justification)), 3))
                } else {
                    Ok((None, 0))
                }
            }

            // Emphase
            b'E' => {
                if data.len() >= 3 {
                    Ok((Some(EscPosCommand::SetEmphasis(data[2] != 0)), 3))
                } else {
                    Ok((None, 0))
                }
            }
            b'F' => Ok((Some(EscPosCommand::SetEmphasis(false)), 2)),

            // Soulignement
            b'-' => {
                if data.len() >= 3 {
                    let underline = data[2];
                    Ok((Some(EscPosCommand::SetUnderline(underline != 0)), 3))
                } else {
                    Ok((None, 0))
                }
            }

            // Italique
            b'4' => Ok((Some(EscPosCommand::SetItalic(true)), 2)),
            b'5' => Ok((Some(EscPosCommand::SetItalic(false)), 2)),

            // Hauteur de ligne
            b'3' => {
                if data.len() >= 3 {
                    let height = data[2] as u32;
                    Ok((Some(EscPosCommand::SetLineHeight(height)), 3))
                } else {
                    Ok((None, 0))
                }
            }

            // Taille de police
            b'!' => {
                if data.len() >= 3 {
                    let size = data[2] as u32;
                    Ok((Some(EscPosCommand::SetFontSize(size)), 3))
                } else {
                    Ok((None, 0))
                }
            }

            // Sélection de la page de code (ESC t n)
            b't' => {
                if data.len() >= 3 {
                    let code_page = data[2];
                    Ok((Some(EscPosCommand::SetCodePage(code_page)), 3))
                } else {
                    Ok((None, 0))
                }
            }

            // Coupe du papier
            b'm' => Ok((Some(EscPosCommand::CutPaper), 2)),
            b'i' => Ok((Some(EscPosCommand::CutPaper), 2)),

            // Alimentation du papier
            b'J' => {
                if data.len() >= 3 {
                    Ok((Some(EscPosCommand::LineFeed), 3))
                } else {
                    Ok((None, 0))
                }
            }

            // Image raster (simplifié)
            b'*' => {
                if data.len() >= 6 {
                    let width = ((data[2] as u16) << 8) | (data[3] as u16);
                    let height = ((data[4] as u16) << 8) | (data[5] as u16);
                    let len = (width * height / 8) as usize;
                    if data.len() >= 6 + len {
                        Ok((Some(EscPosCommand::PrintImage(data[6..6 + len].to_vec())), 6 + len))
                    } else {
                        Ok((None, 0))
                    }
                } else {
                    Ok((None, 0))
                }
            }

            // Commande inconnue
            _ => {
                Ok((Some(EscPosCommand::Unknown(vec![b'\x1B', data[1]])), 2))
            }
        }
    }

    fn parse_gs_command(&self, data: &[u8]) -> Result<(Option<EscPosCommand>, usize)> {
        if data.len() < 2 {
            return Ok((None, 0));
        }

        match data[1] {
            // GS V - Paper Cut
            b'V' => {
                if data.len() < 3 {
                    return Ok((None, 0));
                }
                let m = data[2];
                if m == 0 || m == 1 || m == 48 || m == 49 {
                    // GS V n
                    Ok((Some(EscPosCommand::CutPaper), 3))
                } else if m == 65 || m == 66 {
                    // GS V m n
                    if data.len() < 4 {
                        Ok((None, 0))
                    } else {
                        Ok((Some(EscPosCommand::CutPaper), 4))
                    }
                } else {
                    Ok((Some(EscPosCommand::Unknown(vec![b'\x1D', b'V', m])), 3))
                }
            }

            // Commande GS inconnue
            _ => {
                Ok((Some(EscPosCommand::Unknown(vec![b'\x1D', data[1]])), 2))
            }
        }
    }
}

impl Default for EscPosParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EscPosParser {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            current_code_page: self.current_code_page,
        }
    }
}

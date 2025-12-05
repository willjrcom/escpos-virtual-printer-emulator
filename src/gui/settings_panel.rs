use crate::emulator::EmulatorState;
use egui::Ui;
use std::process::Command;
use std::env;

pub struct SettingsPanel {
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {}
    }
}

impl SettingsPanel {
    pub fn show(&mut self, ui: &mut Ui, _state: &mut EmulatorState) {
        ui.heading("Emulator Settings");
        ui.separator();

        ui.group(|ui| {
            ui.label("Virtual Printer Management");
            ui.label("Installs the emulator as a system printer");
            
            ui.horizontal(|ui| {
                if ui.button("🖨️ Install Printer Driver").clicked() {
                    self.install_printer();
                }

                if ui.button("🗑️ Uninstall Printer").clicked() {
                    self.uninstall_printer();
                }
            });

            ui.label("Note: Requires administrator privileges");
            
            // Check printer status
            if ui.button("🔍 Check Status").clicked() {
                self.check_printer_status();
            }
        });

        ui.separator();

        // Network settings
        ui.group(|ui| {
            ui.label("Network Configuration");
            ui.label("TCP Port: 9100");
            ui.label("Address: 0.0.0.0 (listens on all interfaces)");
            
            if ui.button("📡 Test Connection").clicked() {
                self.test_network_connection();
            }
        });

        ui.separator();

        // Information about operation
        ui.group(|ui| {
            ui.label("ℹ️  Automatic Operation");
            ui.label("• The emulator automatically respects ESC/POS standards");
            ui.label("• Paper width: 50mm, 78mm, 80mm (auto-detection)");
            ui.label("• Font, justification, emphasis: ESC/POS commands");
            ui.label("• No manual configuration needed!");
        });
    }

    fn install_printer(&self) {
        let os = env::consts::OS;
        if os == "windows" {
            self.install_windows_printer();
        } else if os == "linux" {
            self.install_linux_printer();
        } else {
            println!("❌ Unsupported OS for printer installation");
        }
    }

    fn install_windows_printer(&self) {
        // Simplified PowerShell command to avoid syntax errors
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Add-PrinterPort -Name '127.0.0.1:9100' -PrinterHostAddress '127.0.0.1' -PortNumber 9100; \
                 $driver = (Get-PrinterDriver | Where-Object { $_.Name -like '*Microsoft*' } | Select-Object -First 1).Name; \
                 Add-Printer -Name 'ESC_POS_Virtual_Printer' -DriverName $driver -PortName '127.0.0.1:9100'; \
                 Write-Host 'Printer installed successfully'"
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("✅ {}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ Error: {}", stderr);
                }
            }
            Err(e) => {
                println!("❌ Cannot execute printer installation: {}", e);
            }
        }
    }

    fn install_linux_printer(&self) {
    let output = Command::new("bash")
        .args([
            "-c",
            "if command -v lpstat &>/dev/null; then \
                echo 'Installing Linux printer...'; \
                sudo lpadmin -p ESC_POS_Linux_Printer -E -v socket://127.0.0.1:9100 -m raw && \
                sudo lpadmin -d ESC_POS_Linux_Printer && \
                echo 'Linux printer installed successfully!'; \
            else \
                echo 'CUPS not found. Please install CUPS first.'; \
            fi"
        ])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            println!("stdout:\n{}", stdout);
            println!("stderr:\n{}", stderr);

            if !stderr.is_empty() {
                println!("⚠️  CUPS reported some errors — printer may not have been installed.");
            }
        }
        Err(e) => {
            println!("❌ Failed to run installation script: {}", e);
        }
    }
}

    fn uninstall_printer(&self) {
        let os = env::consts::OS;

        let output = if os == "windows" {
            Command::new("powershell")
                .args([
                    "-Command",
                    "Remove-Printer -Name 'ESC_POS_Virtual_Printer' -Confirm:$false; \
                     Remove-PrinterPort -Name '127.0.0.1:9100' -ErrorAction SilentlyContinue; \
                     Write-Host 'Printer uninstalled successfully'"
                ])
                .output()
        } else {

            Command::new("bash")
                .args([
                    "-c",
                    "if command -v lpadmin &>/dev/null; then \
                        echo 'Removing Linux ESC/POS printer...'; \
                        sudo lpadmin -x ESC_POS_Linux_Printer && \
                        echo 'Printer uninstalled successfully'; \
                     else \
                        echo 'CUPS not installed'; \
                     fi"
                ])
                .output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if !stdout.trim().is_empty() {
                    println!("{}", stdout.trim());
                }
                if !stderr.trim().is_empty() {
                    println!("⚠️  {}", stderr.trim());
                }
            }
            Err(e) => {
                println!("❌ Failed to execute uninstallation: {}", e);
            }
        }
    }

    fn check_printer_status(&self) {
        let os = env::consts::OS;

        let output = if os == "windows" {
            Command::new("powershell")
                .args([
                    "-Command",
                    "Get-Printer -Name 'ESC_POS_Virtual_Printer' -ErrorAction SilentlyContinue | Select-Object Name,PortName,DriverName,PrinterStatus"
                ])
                .output()
        } else {
            Command::new("sh")
                .args([
                    "-c",
                    "lpstat -p 2>/dev/null | grep -w 'ESC_POS_Linux_Printer'"
                ])
                .output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);

                if os == "windows" {
                    if stdout.trim().is_empty() {
                        println!("ℹ️  Printer not installed on Windows");
                    } else {
                        println!("✅ Printer installed on Windows:");
                        println!("{}", stdout);
                    }
                } else {
                    if stdout.trim().is_empty() {
                        println!("ℹ️  Printer not installed on Linux");
                    } else {
                        println!("✅ Printer installed on Linux");
                    }
                }
            }

            Err(e) => {
                println!("❌ Printer status check failed: {}", e);
            }
        }
    }

    fn test_network_connection(&self) {
        let os = env::consts::OS;

        let output = if os == "windows" {
            Command::new("powershell")
                .args([
                    "-Command",
                    "Test-NetConnection -ComputerName 127.0.0.1 -Port 9100 -WarningAction SilentlyContinue | Select-Object -ExpandProperty TcpTestSucceeded"
                ])
                .output()
        } else {
            Command::new("sh")
                .args(["-c", "ss -ltn 'sport = :9100'"])
                .output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);

                let port_open = if os == "windows" {
                    stdout.trim().eq_ignore_ascii_case("true")
                } else {
                    !stdout.trim().is_empty()
                };

                if port_open {
                    println!("✅ Port 9100 is open");
                } else {
                    println!("❌ Port 9100 is closed");
                }
            }
            Err(e) => {
                println!("❌ Failed to execute test: {}", e);
            }
        }
    }
}

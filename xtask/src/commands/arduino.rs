use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use xshell::{Shell, cmd};

#[derive(Args)]
pub struct Arduino {
    #[command(subcommand)]
    command: ArduinoCommand,
}

#[derive(Subcommand)]
enum ArduinoCommand {
    /// Build the Arduino firmware
    Build,
    /// Flash the Arduino firmware to the board
    Flash(Flash),
    /// View disassembly of the firmware
    Disasm,
}

#[derive(Args)]
struct Flash {
    /// Serial port to use (auto-detected if not specified)
    #[arg(short, long)]
    port: Option<String>,

    /// Baud rate for flashing
    #[arg(short, long, default_value = "115200")]
    baud: u32,
}

impl Arduino {
    pub fn run(self, sh: &Shell) -> Result<()> {
        match self.command {
            ArduinoCommand::Build => build(sh),
            ArduinoCommand::Flash(opts) => flash(sh, opts),
            ArduinoCommand::Disasm => disasm(sh),
        }
    }
}

fn build(sh: &Shell) -> Result<()> {
    let _guard = sh.push_dir("arduino");

    println!("Building Arduino firmware for AVR ATmega328P...");

    // Check and install AVR toolchain if needed
    ensure_avr_toolchain(sh)?;

    // Check if nightly toolchain is installed
    if cmd!(sh, "rustup toolchain list")
        .read()?
        .lines()
        .all(|line| !line.starts_with("nightly"))
    {
        println!("Installing nightly toolchain...");
        cmd!(sh, "rustup toolchain install nightly").run()?;
    }

    // Check if rust-src is installed
    if cmd!(sh, "rustup component list --toolchain nightly")
        .read()?
        .lines()
        .filter(|line| line.contains("rust-src"))
        .all(|line| !line.contains("(installed)"))
    {
        println!("Installing rust-src component...");
        cmd!(sh, "rustup component add rust-src --toolchain nightly").run()?;
    }

    // Build with nightly
    println!("Building with cargo +nightly...");
    cmd!(sh, "cargo +nightly build --release").run()?;

    // Convert ELF to HEX
    println!("Converting ELF to HEX format...");
    let elf_path = "../target/avr-none/release/led-star-arduino.elf";
    let hex_path = "../target/avr-none/release/led-star-arduino.hex";

    cmd!(sh, "avr-objcopy -O ihex {elf_path} {hex_path}")
        .run()
        .context("Failed to run avr-objcopy. Make sure AVR toolchain is installed.")?;

    // Show binary size
    println!("\nBinary size:");
    cmd!(sh, "avr-size {elf_path}").run()?;

    println!("\n✓ Build complete!");
    println!("  ELF: arduino/{}", elf_path);
    println!("  HEX: arduino/{}", hex_path);

    Ok(())
}

fn flash(sh: &Shell, flash_opts: Flash) -> Result<()> {
    let _guard = sh.push_dir("arduino");

    let hex_path = PathBuf::from("../target/avr-none/release/led-star-arduino.hex");

    // Check if hex file exists
    if !sh.path_exists(&hex_path) {
        println!("HEX file not found. Building first...");
        build(sh)?;
    }

    // Detect or use specified port
    let port = if let Some(p) = flash_opts.port {
        p
    } else {
        detect_arduino_port(sh)?
    };

    println!("Flashing to Arduino Uno on {}...", port);
    println!("Using baud rate: {}", flash_opts.baud);

    let hex_str = hex_path.to_str().unwrap();
    let baud_str = flash_opts.baud.to_string();

    cmd!(
        sh,
        "avrdude -p atmega328p -c arduino -P {port} -b {baud_str} -U flash:w:{hex_str}:i"
    )
    .run()
    .context("Failed to flash. Make sure avrdude is installed and the Arduino is connected.")?;

    println!("\n✓ Flash complete!");

    Ok(())
}

fn disasm(sh: &Shell) -> Result<()> {
    let _guard = sh.push_dir("arduino");

    let elf_path = PathBuf::from("../target/avr-none/release/led-star-arduino.elf");

    // Check if elf file exists
    if !sh.path_exists(&elf_path) {
        println!("ELF file not found. Building first...");
        build(sh)?;
    }

    println!("Disassembling firmware...\n");

    let elf_str = elf_path.to_str().unwrap();

    // Show full disassembly with source intermixed if available
    cmd!(sh, "avr-objdump -d -S {elf_str}")
        .run()
        .context("Failed to run avr-objdump. Make sure AVR toolchain is installed.")?;

    Ok(())
}

fn ensure_avr_toolchain(sh: &Shell) -> Result<()> {
    // Check if avr-gcc is already installed
    if cmd!(sh, "which avr-gcc").read().is_ok() {
        println!("✓ AVR toolchain found");
        return Ok(());
    }

    println!("AVR toolchain not found. Installing...");

    if cfg!(target_os = "macos") {
        // Check if Homebrew is installed
        if cmd!(sh, "which brew").read().is_err() {
            bail!(
                "Homebrew not found. Please install Homebrew first:\n\
                 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
            );
        }

        println!("Installing AVR toolchain via Homebrew...");
        println!("This may take a few minutes...");

        // Tap the osx-cross/avr repository for AVR toolchain
        let _ = cmd!(sh, "brew tap osx-cross/avr").run(); // Don't fail if already tapped

        // Install the AVR toolchain
        cmd!(sh, "brew install avr-gcc avrdude")
            .run()
            .context("Failed to install AVR toolchain")?;
    } else if cfg!(target_os = "linux") {
        // Try to detect if we can use sudo
        if cmd!(sh, "which sudo").read().is_ok() {
            println!("Installing AVR toolchain via apt...");
            println!("You may be prompted for your password.");

            cmd!(sh, "sudo apt-get update").run().ok(); // Don't fail if update fails
            cmd!(sh, "sudo apt-get install -y gcc-avr avr-libc avrdude")
                .run()
                .context("Failed to install AVR toolchain")?;
        } else {
            bail!(
                "Cannot install AVR toolchain automatically.\n\
                 Please install manually: apt-get install gcc-avr avr-libc avrdude"
            );
        }
    } else {
        bail!(
            "Automatic installation not supported on this OS. Please install avr-gcc and avrdude manually."
        );
    }

    // Verify installation
    if cmd!(sh, "which avr-gcc").read().is_err() {
        bail!("AVR toolchain installation failed. Please install manually.");
    }

    println!("✓ AVR toolchain installed successfully");
    Ok(())
}

fn detect_arduino_port(sh: &Shell) -> Result<String> {
    println!("Auto-detecting Arduino port...");

    // Try common patterns for macOS and Linux
    let patterns = if cfg!(target_os = "macos") {
        vec!["/dev/tty.usb*", "/dev/cu.usb*", "/dev/tty.wchusb*"]
    } else {
        vec!["/dev/ttyACM*", "/dev/ttyUSB*"]
    };

    for pattern in patterns {
        // Use ls to find matching ports
        if let Ok(output) = cmd!(sh, "sh -c")
            .arg(format!("ls {pattern} 2>/dev/null"))
            .read()
        {
            let ports: Vec<&str> = output.lines().collect();
            if !ports.is_empty() {
                let port = ports[0].to_string();
                println!("Found Arduino at: {}", port);
                return Ok(port);
            }
        }
    }

    bail!(
        "Could not detect Arduino port. Please specify with --port.\n\
         Common ports:\n\
         - macOS: /dev/tty.usbserial-*, /dev/cu.usbserial-*\n\
         - Linux: /dev/ttyACM*, /dev/ttyUSB*"
    )
}

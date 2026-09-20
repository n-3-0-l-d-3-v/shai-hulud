//! `artifact` — pack a Machine program into a PDF/SVG, or explicitly load
//! and run one from such a file inside the sandboxed VM.

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use artifact::Format;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "artifact", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Pack a `.imcs` assembly program into a `.pdf` or `.svg` container.
    Pack {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "Impossible Computer artifact")]
        title: String,
    },
    /// Verify the container and print the embedded program's disassembly.
    Inspect { input: PathBuf },
    /// Verify the container, then run the embedded program in the sandbox VM.
    Run {
        input: PathBuf,
        #[arg(long, default_value_t = 1 << 20)]
        mem_size: usize,
        #[arg(long, default_value_t = 10_000_000)]
        step_budget: u64,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Pack {
            input,
            output,
            title,
        } => {
            let src = fs::read_to_string(&input).with_context(|| input.display().to_string())?;
            let program = asm::assemble(&src).map_err(|e| anyhow!(e))?;
            let format = match output.extension().and_then(|e| e.to_str()) {
                Some("pdf") => Format::Pdf,
                Some("svg") => Format::Svg,
                _ => bail!("output must end in .pdf or .svg"),
            };
            let bytes = artifact::embed(format, &program, &title)?;
            fs::write(&output, &bytes)?;
            println!("wrote {} ({} bytes)", output.display(), bytes.len());
            Ok(())
        }
        Command::Inspect { input } => {
            let program = artifact::extract(&fs::read(&input)?)?;
            print!("{}", isa::disasm::disassemble(&program));
            Ok(())
        }
        Command::Run {
            input,
            mem_size,
            step_budget,
        } => {
            let program = artifact::extract(&fs::read(&input)?)?;
            let report = artifact::run(program, mem_size, step_budget);
            for line in &report.output {
                println!("{line}");
            }
            match report.exit {
                Ok(exit) => {
                    eprintln!("exit: {exit:?}");
                    Ok(())
                }
                Err(trap) => bail!("trapped: {trap}"),
            }
        }
    }
}

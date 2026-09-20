//! Data-format containers for Machine programs.
//!
//! A program is embedded as inert data in an ordinary PDF (file attachment)
//! or SVG (`<metadata>` element). The files are valid, harmless documents in
//! any normal viewer: nothing executes unless *this* loader is explicitly
//! invoked on them. The loader is strict and fails closed: exactly one
//! payload, size-capped, SHA-256 verified, program validated, and executed in
//! the Machine VM with a memory cap and step budget (the VM has no host I/O
//! beyond exit and a captured output buffer).

pub mod pdf;
pub mod svg;

use sha2::{Digest, Sha256};

/// Hard cap on payload size accepted by the loader.
pub const MAX_PAYLOAD: usize = 1 << 20;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("no embedded payload found")]
    NotFound,
    #[error("more than one embedded payload (refusing ambiguous container)")]
    Multiple,
    #[error("malformed container: {0}")]
    Malformed(String),
    #[error("payload too large: {0} bytes")]
    TooLarge(usize),
    #[error("payload integrity check failed")]
    HashMismatch,
    #[error("invalid program: {0}")]
    Program(String),
    #[error("unrecognised container format")]
    UnknownFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Pdf,
    Svg,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn detect(bytes: &[u8]) -> Option<Format> {
    if bytes.starts_with(b"%PDF-") {
        Some(Format::Pdf)
    } else if bytes.windows(4).take(512).any(|w| w == b"<svg") {
        Some(Format::Svg)
    } else {
        None
    }
}

/// Serialise a program and wrap it in the requested container.
pub fn embed(format: Format, program: &isa::Program, title: &str) -> Result<Vec<u8>, Error> {
    program
        .validate()
        .map_err(|e| Error::Program(e.to_string()))?;
    let payload = serde_json::to_vec(program).map_err(|e| Error::Program(e.to_string()))?;
    if payload.len() > MAX_PAYLOAD {
        return Err(Error::TooLarge(payload.len()));
    }
    match format {
        Format::Pdf => pdf::wrap(&payload, title),
        Format::Svg => svg::wrap(&payload, title),
    }
}

/// Extract, verify and validate the program inside a container.
pub fn extract(bytes: &[u8]) -> Result<isa::Program, Error> {
    let payload = match detect(bytes).ok_or(Error::UnknownFormat)? {
        Format::Pdf => pdf::unwrap(bytes)?,
        Format::Svg => svg::unwrap(bytes)?,
    };
    let program: isa::Program =
        serde_json::from_slice(&payload).map_err(|e| Error::Program(e.to_string()))?;
    program
        .validate()
        .map_err(|e| Error::Program(e.to_string()))?;
    Ok(program)
}

pub struct RunReport {
    pub output: Vec<String>,
    pub exit: Result<vm::ExitReason, vm::Trap>,
}

/// Run a program under explicit resource limits.
pub fn run(program: isa::Program, mem_size: usize, step_budget: u64) -> RunReport {
    let mut machine = vm::Vm::new(program)
        .with_memory_size(mem_size)
        .with_step_budget(step_budget);
    let exit = machine.run();
    RunReport {
        output: machine.output.clone(),
        exit,
    }
}

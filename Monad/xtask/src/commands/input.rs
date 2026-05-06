//! Interactive input helpers for xtask auth commands.
#![warn(clippy::pedantic)]

use std::io::{self, Write};

use anyhow::{Context, Result};

pub fn prompt_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush().context("failed to flush stdout")?;

    let mut value = String::new();
    let n = io::stdin()
        .read_line(&mut value)
        .context("failed to read from stdin")?;

    if n == 0 {
        anyhow::bail!("unexpected end of input — this command requires an interactive terminal");
    }

    Ok(value.trim().to_string())
}

pub fn prompt_secret(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush().context("failed to flush stdout")?;

    let secret = rpassword::read_password().context("failed to read secret input")?;
    Ok(secret)
}

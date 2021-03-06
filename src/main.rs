use error_iter::ErrorIter;
use picard::{decode, Error};
use std::{
    env,
    fs::{self, File},
    io::Write as _,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use thiserror::Error;

#[derive(Debug, Error)]
enum RunError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Decoding failed")]
    Decode(#[from] Error),

    #[error("Could not parse command line arguments\n  {0}")]
    ArgParse(&'static str),
}

impl ErrorIter for RunError {}

fn main() -> Result<(), RunError> {
    if let Err(err) = run() {
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);

        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(&mut stderr, "Error: {}", err)?;
        for source in err.chain().skip(1) {
            writeln!(&mut stderr, "  Caused by: {}", source)?;
        }
        stderr.reset()?;

        eprintln!();
    }

    Ok(())
}

fn run() -> Result<(), RunError> {
    println!("picard: PIC Action Replay Decoder");
    println!("Make it so!");
    println!();

    let mut args = env::args_os();
    let input_path = args.nth(1).ok_or(RunError::ArgParse(
        "Usage: picard <input_path> [output_path]",
    ))?;
    let output_path = args.next();

    let input = fs::read(input_path)?;
    let output = output_path.as_ref().map(File::create).transpose()?;

    let mut data = [0; 4];

    for slice in input.chunks_exact(8) {
        if output.is_none() {
            print!("recv: ");
            for b in slice {
                print!("{:#04x} ", b);
            }
            println!();
        }

        decode(&mut data, slice)?;

        if let Some(mut output) = output.as_ref() {
            output.write_all(&data)?
        } else {
            print!("send: ");
            for b in &data {
                print!("{:#04x} ", b);
            }
            println!();

            println!();
        }
    }

    if let Some(output_path) = output_path {
        println!("Responses written to {:?}", output_path);
    }

    Ok(())
}

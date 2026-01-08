use std::{
    env::{self, VarError},
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

use grass::{Options, OutputStyle};
use grass_compiler::Error as SassError;
use thiserror::Error;

fn main() {
    if let Err(error) = built::write_built_file() {
        panic!("failed to acquire build-time information: {}", error);
    }

    if let Err(error) = compile_stylesheet() {
        panic!("{error}");
    }
}

#[derive(Debug, Error)]
enum CompileStylesheetError {
    #[error("failed to compile SCSS: {0}")]
    CompileSass(#[from] Box<SassError>),

    #[error("could not read value of env var {0}: {1}")]
    Var(&'static str, #[source] VarError),

    #[error("could not create stylesheet output file {0}: {1}")]
    CreateOutFile(PathBuf, #[source] io::Error),

    #[error("could not write compiled CSS to {0}: {1}")]
    WriteCss(PathBuf, #[source] io::Error),
}

fn compile_stylesheet() -> Result<(), CompileStylesheetError> {
    println!("cargo:rerun-if-changed=scss/");
    let compiled_css = grass::from_path(
        "scss/style.scss",
        &Options::default().style(OutputStyle::Compressed),
    )?;

    let mut out_path: PathBuf = env::var("OUT_DIR")
        .map_err(|err| CompileStylesheetError::Var("OUT_DIR", err))?
        .into();
    out_path.push("style.css");

    let mut stylesheet = File::create(&out_path)
        .map_err(|err| CompileStylesheetError::CreateOutFile(out_path.clone(), err))?;

    write!(stylesheet, "{}", compiled_css)
        .map_err(|err| CompileStylesheetError::WriteCss(out_path, err))
}

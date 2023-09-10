mod ascii;
mod cli;
mod math;
mod video;

use crate::cli::CLIArgs;
use ascii::LogDisplay;
use std::time::Instant;

macro_rules! unwrap {
  (Ok $wrapped: expr, Err $( $err: expr ),*) => {
    match $wrapped {
      Ok(v) => v,
      Err(e) => {
        log!(err@$( $err ),*);
        log!(err@"{e}");
        return;
      }
    }
  };
}

fn main() {
  let args = unwrap!(Ok CLIArgs::read(), Err "Error");

  let mut decoder = unwrap!(
    Ok video::quicktime::QTDecoder::new(&args.filepath),
    Err "QTDecoder could not open file"
  );

  let start_time = Instant::now();
  let atoms = decoder.decode();
  let end_time = Instant::now();
  if args.debug {
    log!("ATOMS => {atoms:#?}");
  }

  log!(ok@"Done in {:?}", end_time - start_time);
}

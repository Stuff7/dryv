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

  let start_time = Instant::now();
  let video = unwrap!(
    Ok video::Video::open(&args.filepath),
    Err "Could not open video"
  );
  let end_time = Instant::now();
  if args.debug {
    log!("{video}");
  }

  log!(ok@"Done in {:?}", end_time - start_time);
}

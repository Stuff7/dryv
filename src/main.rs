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

  let mut mp4_parser = unwrap!(
    Ok video::mp4::MP4Decoder::new(&args.filepath),
    Err "MP4Decoder could not open file"
  );

  let start_time = Instant::now();
  unwrap!(Ok mp4_parser.decode(), Err "Could not decode MP4 file");
  let end_time = Instant::now();

  log!(ok@"Done in {:?}", end_time - start_time);
}

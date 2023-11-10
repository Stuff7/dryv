#![allow(clippy::needless_range_loop)]
mod ascii;
mod byte;
mod cli;
mod display;
mod math;
mod video;

use crate::cli::CLIArgs;
use ascii::LogDisplay;
use std::{
  fs::File,
  ptr,
  sync::atomic::{AtomicPtr, Ordering},
  time::Instant,
};

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

static LOG_FILE_PTR: AtomicPtr<File> = AtomicPtr::new(ptr::null_mut());

fn main() {
  let args = unwrap!(Ok CLIArgs::read(), Err "Error");

  let mut log_file = unwrap!(Ok File::create("debug.log"), Err "Could not create log file");
  LOG_FILE_PTR.store(&mut log_file as *mut _, Ordering::SeqCst);

  let start_time = Instant::now();
  let video = unwrap!(
    Ok video::Video::open(&args.filepath),
    Err "Could not open video"
  );
  let end_time = Instant::now();
  if args.debug {
    println!("{video}");
  }

  log!(ok@"Done in {:?}", end_time - start_time);
}

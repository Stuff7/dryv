use std::fmt::Display;

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const UNDERLINE: &str = "\x1b[4m";

pub trait Color {
  fn rgb(&self, r: u8, g: u8, b: u8) -> String
  where
    Self: Display,
  {
    format!("\x1b[38;2;{r};{g};{b}m{self}")
  }
  fn on_rgb(&self, r: u8, g: u8, b: u8) -> String
  where
    Self: Display,
  {
    format!("\x1b[48;2;{r};{g};{b}m{self}")
  }
  fn bold(&self) -> String
  where
    Self: Display,
  {
    format!("{BOLD}{self}")
  }
  fn underline(&self) -> String
  where
    Self: Display,
  {
    format!("{UNDERLINE}{self}")
  }
  fn reset(&self) -> String
  where
    Self: Display,
  {
    format!("{self}{RESET}")
  }
}

impl Color for String {}
impl<'a> Color for &'a str {}

pub trait LogDisplay: Color {
  fn err(&self) -> String
  where
    Self: Display,
  {
    format!(
      "{}[ERROR]{RESET} {}\n",
      "".bold().underline().rgb(255, 75, 75),
      self.rgb(255, 105, 105).reset()
    )
  }
  fn ok(&self) -> String
  where
    Self: Display,
  {
    format!(
      "{}[OK]{RESET} {}\n",
      "".bold().underline().rgb(0, 255, 94),
      self.rgb(0, 255, 155).reset()
    )
  }
  fn info(&self) -> String
  where
    Self: Display,
  {
    format!(
      "{}[INFO]{RESET} {}\n",
      "".bold().underline().rgb(240, 105, 255),
      self.rgb(250, 155, 255).reset()
    )
  }
  fn warn(&self) -> String
  where
    Self: Display,
  {
    format!(
      "{}[WARN]{RESET} {}\n",
      "".bold().underline().rgb(255, 255, 105),
      self.rgb(255, 255, 155).reset()
    )
  }
  fn log(&self) -> String
  where
    Self: Display,
  {
    format!(
      "{}[LOG] {}\n",
      "".rgb(255, 253, 194),
      self.rgb(255, 253, 194).reset()
    )
  }
}

impl LogDisplay for String {}
impl<'a> LogDisplay for &'a str {}

#[macro_export]
macro_rules! log {
  ( $($fn: ident).* @ $($t: tt)* ) => {
    {
      let msg = format!($($t)*).$($fn()).*;
      print!("{}", msg);
    }
  };
  ( $($t: tt)* ) => {
    {
      let msg = format!($($t)*).log();
      print!("{}", msg);
    }
  };
}

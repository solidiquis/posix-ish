use crate::file::{FileInfo, FileType};
use posix_ish_utils::{
    error::{Result, ToLsResult},
    tty,
};
use std::env;

/// https://gist.github.com/thomd/7667642
pub trait Colorizer {
    /// Returns the stylized string and how many characters were added in total for the
    /// ANSI-escapes for the file name.
    fn name(&self, file: &FileInfo) -> (String, usize);
}

pub fn init_colorizer() -> Result<Box<dyn Colorizer>> {
    if !tty::enable_color() {
        return Ok(Box::new(NoColor));
    }
    let Some(ls_colors) = env::var_os("LS_COLORS") else {
        return Ok(Box::new(NoColor));
    };

    let mut color = LsColor::default();

    for attr in ls_colors.to_string_lossy().split(':') {
        let mut pair = attr.split("=");
        let ft = pair.next().io_error("malformed LS_COLORS env variable")?;
        let val = pair.next().io_error("malformed LS_COLORS env variable")?;

        match ft {
            "di" => color.di = val.to_string(),
            "fi" => color.fi = val.to_string(),
            "ln" => color.ln = val.to_string(),
            "pi" => color.pi = val.to_string(),
            "so" => color.so = val.to_string(),
            "bd" => color.bd = val.to_string(),
            "cd" => color.cd = val.to_string(),
            "or" => color.or = val.to_string(),
            "mi" => color.mi = val.to_string(),
            "ex" => color.ex = val.to_string(),
            _ => continue,
        }
    }

    Ok(Box::new(color))
}

struct LsColor {
    /// Directory
    di: String,
    /// File
    fi: String,
    /// Link
    ln: String,
    /// Fifo
    pi: String,
    /// Socket
    so: String,
    /// Block device
    bd: String,
    /// Character device
    cd: String,
    /// Orphan link
    or: String,
    /// Non-existent file pointed to be link.. not quite sure
    /// how this is different from an orphan link tbh.
    mi: String,
    /// Executable
    ex: String,
}

struct NoColor;

impl Colorizer for NoColor {
    fn name(&self, file: &FileInfo) -> (String, usize) {
        (file.name.clone(), 0)
    }
}

impl Colorizer for LsColor {
    fn name(&self, file: &FileInfo) -> (String, usize) {
        const BASE_NUM_ANSI_CHARS: usize = 7;

        let file_name = &file.name;

        macro_rules! color {
            ($ft:expr, $name:expr) => {{
                let num_ansi_chars = BASE_NUM_ANSI_CHARS + $ft.len();
                (format!("\x1b[{}m{}\x1b[0m", $ft, $name), num_ansi_chars)
            }};
        }

        if file.is_executable() && !matches!(file.file_type, FileType::Dir | FileType::SymLink) {
            return color!(self.ex, file_name);
        }

        match file.file_type {
            FileType::Dir => color!(self.di, file_name),
            FileType::File => color!(self.fi, file_name),
            FileType::SymLink if file.referent.is_none() => color!(self.or, file_name),
            FileType::SymLink => color!(self.ln, file_name),
            FileType::Fifo => color!(self.pi, file_name),
            FileType::Socket => color!(self.so, file_name),
            FileType::CharDev => color!(self.cd, file_name),
            FileType::BlockDev => color!(self.bd, file_name),
            FileType::Unknown => color!(self.ex, file_name),
        }
    }
}

impl Default for LsColor {
    fn default() -> Self {
        Self {
            di: String::from("0"),
            fi: String::from("0"),
            ln: String::from("0"),
            pi: String::from("0"),
            so: String::from("0"),
            bd: String::from("0"),
            cd: String::from("0"),
            or: String::from("0"),
            mi: String::from("0"),
            ex: String::from("0"),
        }
    }
}

use crate::error::{Error, Result, ToLsResult};
use std::{
    env::{ArgsOs, current_dir},
    ffi::OsString,
    fmt,
    io::{IsTerminal, stdout},
    path::PathBuf,
};

/// https://man7.org/linux/man-pages/man1/ls.1.html
/// https://www.unix.com/man_page/linux/1posix/ls/
const HELP: &str = r#"
#BIN_NAME #VERSION

USAGE:
  #BIN_NAME [OPTIONS] [PATH ...]

POSITIONAL ARGUMENTS:
  <PATH>...   Files to operate on. If empty then defaults to current working directory.

BASIC OPTIONS [-afq]:
  -a          Write out all directory entries, including those whose names begin with a period ( '.' ).
              Entries beginning with a period shall not be written out unless explicitly referenced.
  -f          Force  each  argument  to be interpreted as a directory and list the name found in each slot.
              This option shall turn off -l, -t, -s, and -r, and shall turn on -a; the order is the order in
              which entries appear in the directory.
  -q          Force  each instance of non-printable filename characters and <tab>s to be written as the
              question-mark ('?') character. Enabled by default if stdout is a terminal.

ADDITIONAL OUTPUT INFO OPTIONS [-Fips]:
  -F          Write a slash ('/') immediately after each  pathname  that  is a directory, an asterisk ('*')
              after each that is executable, a vertical bar ('|') after each that is a FIFO, and an at sign ('@')
              after each that is a symbolic link.
  -i          For each file, write the file's file serial number.
  -p          Write a slash ('/') after each filename if that file is a directory.
  -s          Indicate the total number of file system blocks consumed by each file displayed (1 block = 512 bytes).

SORTING OPTIONS [-r] [-c | -u | -t]:
  -r          Reverse the order of the sort.
  -t          Sort with the primary key being time modified (most recently modified first) and the secondary
              key being filename.
  -c          Like -t but uses last modification of the file status information.
  -u          Like -t but uses time of last access.
  -X          Sort entries alphabetically.

FOLLOW LINK OPTIONS [-H | -L]:
  -H          Symbolic links on the command line are followed.
  -L          Follow all symbolic links to final target and list the file or directory the link references rather
              than the link itself.

DIRECTORY OPTIONS [-d | -R]:
  -d          Do not follow symbolic links named as operands unless the -H or -L options are specified. Do not treat
              directories differently than other types of files.
  -R          Recursively list subdirectories encountered.

OUTPUT FORMAT [-nlog | -m | -C | -x | -1]:
  -l          (ell) Do not follow symbolic links named as operands unless the -H or -L options are specified.
              Write out in long format.
  -o          The same as -l (ell), except that the group shall not be written.
  -g          The same as -l (ell), except that the owner shall not be written.
  -n          The same as -l (ell), except that the owner's UID and GID numbers shall be written, rather than
              the associated character strings.
  -m          Comma separated output.
  -C          Write multi-text-column output with entries sorted down the columns.
  -x          The same as -C, except that the multi-text-column output is produced with entries sorted across,
              rather than down, the columns.
  -1          (Number one) Force output to be one entry per line.
"#;

/// https://www.unix.com/man_page/posix/1posix/ls/
#[derive(Default)]
pub struct ProgramBehavior {
    /// `-a`
    pub include_all: IncludeAll,
    /// Long options
    pub output_format: OutputFormat,
    /// `-R`
    pub recursive_dir_walk: bool,
    /// `-t`, `-u`, `-c`, `-S` and `-r`
    pub sort: Sort,
    /// `-H` and `-L`
    pub follow_links: FollowLinks,
    /// `-F`
    pub include_file_type_symbol: bool,
    /// `-d`
    pub treat_dir_operands_as_regular_files: bool,
    /// `-f`
    pub treat_operands_as_dir_no_order: bool,
    /// `-i`
    pub include_file_serial_number: bool,
    /// `-p`
    pub append_fslash_to_dir: bool,
    /// `-q`
    pub non_printable_and_tabs_to_qmark: bool,
    /// `-r`
    pub reverse_sort: bool,
    /// `-h`, `--human-readable
    pub human_readable_size: bool,
    /// `-s`
    pub include_block_size: bool,
    /// `--si`
    pub si_units: bool,
    /// `--help`
    pub show_help: bool,
}

#[derive(Default)]
pub enum OutputFormat {
    /// `-C`
    #[default]
    MultiColumn,
    /// `-x`
    MultiColumnHorizontalSort,
    /// `-m`
    CommaSeparated,
    /// `-1`
    OneEntryPerLine,
    /// Long output options
    Long(Long),
}

#[derive(Default, PartialEq, Eq)]
pub struct Long {
    pub exclude_owner: bool,
    pub exclude_group: bool,
    pub owner_group_id: bool,
}

#[derive(Default)]
pub enum IncludeAll {
    /// Exclude files that begin with '.'
    #[default]
    ExcludeHidden,
    /// `-a`
    All,
}

#[derive(Default, Eq, PartialEq)]
pub enum FollowLinks {
    /// Don't follow links
    #[default]
    NoFollow,
    /// `-L`
    All,
    /// `-H`
    ArgsOnly,
}

#[derive(Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    None,
    /// `-S`
    Size,
    /// `-t`
    Mod,
    /// `-u`
    Access,
    /// `-c`
    Status,
}

/// For exhaustive matching
#[derive(Debug, PartialEq, Eq)]
pub enum Opt {
    LowerA,
    LowerC,
    LowerD,
    LowerF,
    LowerG,
    LowerH,
    LowerI,
    LowerL,
    LowerM,
    LowerN,
    LowerO,
    LowerP,
    LowerQ,
    LowerR,
    LowerS,
    LowerT,
    LowerU,
    LowerX,
    UpperC,
    UpperF,
    UpperH,
    UpperL,
    UpperR,
    UpperS,
    Number1,
}

const LONG_OPTS: [Opt; 4] = [Opt::LowerN, Opt::LowerL, Opt::LowerO, Opt::LowerG];

pub enum LongOpt {
    /// `--help`
    Help,
    /// `--si`
    Si,
    /// `--human-readable`
    HumanReadableSize,
}

impl TryFrom<char> for Opt {
    type Error = Error;

    fn try_from(opt: char) -> Result<Self> {
        match opt {
            'a' => Ok(Self::LowerA),
            'c' => Ok(Self::LowerC),
            'd' => Ok(Self::LowerD),
            'f' => Ok(Self::LowerF),
            'g' => Ok(Self::LowerG),
            'h' => Ok(Self::LowerH),
            'i' => Ok(Self::LowerI),
            'l' => Ok(Self::LowerL),
            'm' => Ok(Self::LowerM),
            'n' => Ok(Self::LowerN),
            'o' => Ok(Self::LowerO),
            'p' => Ok(Self::LowerP),
            'q' => Ok(Self::LowerQ),
            'r' => Ok(Self::LowerR),
            's' => Ok(Self::LowerS),
            't' => Ok(Self::LowerT),
            'u' => Ok(Self::LowerU),
            'x' => Ok(Self::LowerX),
            'C' => Ok(Self::UpperC),
            'F' => Ok(Self::UpperF),
            'H' => Ok(Self::UpperH),
            'L' => Ok(Self::UpperL),
            'R' => Ok(Self::UpperR),
            'S' => Ok(Self::UpperS),
            '1' => Ok(Self::Number1),
            _ => Error::invalid_argument(format!("invalid option '-{opt}'")).into(),
        }
    }
}

impl fmt::Display for Opt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LowerA => write!(f, "-a"),
            Self::LowerC => write!(f, "-c"),
            Self::LowerD => write!(f, "-d"),
            Self::LowerF => write!(f, "-f"),
            Self::LowerG => write!(f, "-g"),
            Self::LowerH => write!(f, "-h"),
            Self::LowerI => write!(f, "-i"),
            Self::LowerL => write!(f, "-l"),
            Self::LowerM => write!(f, "-m"),
            Self::LowerN => write!(f, "-n"),
            Self::LowerO => write!(f, "-o"),
            Self::LowerP => write!(f, "-p"),
            Self::LowerQ => write!(f, "-q"),
            Self::LowerR => write!(f, "-r"),
            Self::LowerS => write!(f, "-s"),
            Self::LowerT => write!(f, "-t"),
            Self::LowerU => write!(f, "-u"),
            Self::LowerX => write!(f, "-x"),
            Self::UpperC => write!(f, "-C"),
            Self::UpperF => write!(f, "-F"),
            Self::UpperH => write!(f, "-H"),
            Self::UpperL => write!(f, "-L"),
            Self::UpperR => write!(f, "-R"),
            Self::UpperS => write!(f, "-S"),
            Self::Number1 => write!(f, "-1"),
        }
    }
}

impl fmt::Display for LongOpt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Help => write!(f, "--help"),
            Self::HumanReadableSize => write!(f, "--human-readable"),
            Self::Si => write!(f, "--si"),
        }
    }
}

impl TryFrom<&str> for LongOpt {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "--help" => Ok(Self::Help),
            "--si" => Ok(Self::Si),
            "--human-readable" => Ok(Self::HumanReadableSize),
            _ => Error::invalid_argument(format!("invalid option '{value}'")).into(),
        }
    }
}

impl TryFrom<&Opt> for OutputFormat {
    type Error = Error;

    fn try_from(opt: &Opt) -> Result<Self> {
        match opt {
            Opt::LowerL => Ok(OutputFormat::Long(Long::default())),
            Opt::LowerG => Ok(OutputFormat::Long(Long {
                exclude_owner: true,
                ..Default::default()
            })),
            Opt::LowerN => Ok(OutputFormat::Long(Long {
                owner_group_id: true,
                ..Default::default()
            })),
            Opt::LowerO => Ok(OutputFormat::Long(Long {
                exclude_group: true,
                ..Default::default()
            })),
            Opt::LowerM => Ok(OutputFormat::CommaSeparated),
            Opt::LowerX => Ok(OutputFormat::MultiColumnHorizontalSort),
            Opt::UpperC => Ok(OutputFormat::MultiColumn),
            Opt::Number1 => Ok(OutputFormat::OneEntryPerLine),
            _ => unreachable!("encountered an unknown short option"),
        }
    }
}

pub fn parse(args: ArgsOs) -> Result<(Vec<OsString>, ProgramBehavior)> {
    let mut operands = Vec::new();
    let mut behavior = ProgramBehavior::default();
    let stdout_is_tty = stdout().is_terminal();

    // tty specific defaults
    if stdout_is_tty {
        behavior.non_printable_and_tabs_to_qmark = true;
    }

    let mut done_parsing_options = false;

    let mut disable_long = false;
    let mut disable_sort = false;
    let mut disable_size = false;

    // Guards for mutually exclusive options
    let mut sort_pkey = None;
    let mut output_format = None;

    // Skip program name
    for arg in args.into_iter().skip(1) {
        let arg_utf8_lossy = arg.to_string_lossy();
        let is_opt = arg_utf8_lossy.starts_with("-");
        let is_long_opt = arg_utf8_lossy.starts_with("--");

        if (!is_opt && !is_long_opt) || done_parsing_options {
            done_parsing_options = true;
            operands.push(arg);
            continue;
        }

        let arg_string = arg
            .into_string()
            .ok()
            .invalid_argument("options must strictly be UTF-8")?;

        if is_long_opt {
            let long_opt = LongOpt::try_from(arg_string.as_str())?;

            match long_opt {
                LongOpt::Help => {
                    behavior.show_help = true;
                    return Ok((Vec::new(), behavior));
                }
                LongOpt::Si => {
                    behavior.si_units = true;
                }
                LongOpt::HumanReadableSize => {
                    behavior.human_readable_size = true;
                }
            }
        } else if is_opt {
            for opt_char in arg_string.chars().skip(1) {
                let option = Opt::try_from(opt_char)?;

                match option {
                    // BASIC OPTIONS (BEGIN)
                    Opt::LowerA => {
                        behavior.include_all = IncludeAll::All;
                    }
                    Opt::LowerF => {
                        behavior.treat_operands_as_dir_no_order = true;
                        behavior.include_all = IncludeAll::All;
                        behavior.sort = Sort::None;
                        behavior.include_block_size = false;

                        disable_long = true;
                        disable_size = true;
                        disable_sort = true;

                        if let OutputFormat::Long(_) = behavior.output_format {
                            behavior.output_format = OutputFormat::default();
                        }
                    }
                    Opt::LowerQ => {
                        behavior.non_printable_and_tabs_to_qmark = true;
                    }
                    Opt::LowerH => {
                        behavior.human_readable_size = true;
                    }
                    // BASIC OPTIONS (END)

                    // ADDITIONAL OUTPUT INFO OPTIONS (BEGIN)
                    Opt::UpperF => {
                        behavior.include_file_type_symbol = true;
                    }
                    Opt::LowerI => {
                        behavior.include_file_serial_number = true;
                    }
                    Opt::LowerP => {
                        behavior.append_fslash_to_dir = true;
                    }
                    Opt::LowerS => {
                        if !disable_size {
                            behavior.include_block_size = true;
                        }
                    }
                    // ADDITIONAL OUTPUT INFO OPTIONS (END)

                    // SORTING
                    Opt::LowerR => {
                        behavior.reverse_sort = true;
                    }

                    // GROUP: Sorting (BEGIN)
                    Opt::LowerC | Opt::LowerU | Opt::LowerT | Opt::UpperS => {
                        if disable_sort {
                            continue;
                        }
                        if let Some(pkey) = sort_pkey
                            && pkey != option
                        {
                            let msg = format!("`{option}` cannot but used with `{pkey}`");
                            return Error::invalid_argument(msg).into();
                        } else {
                            behavior.sort = match option {
                                Opt::LowerC => Sort::Status,
                                Opt::LowerU => Sort::Access,
                                Opt::LowerT => Sort::Mod,
                                Opt::UpperS => Sort::Size,
                                _ => unreachable!(),
                            };
                            sort_pkey = Some(option);
                        }
                    }
                    // GROUP: Sorting (END)

                    // GROUP: Follow links (BEGIN)
                    Opt::UpperH => {
                        if let FollowLinks::NoFollow = behavior.follow_links {
                            behavior.follow_links = FollowLinks::ArgsOnly;
                        } else {
                            let msg = format!("`{option}` cannot but used with `{}`", Opt::UpperL);
                            return Error::invalid_argument(msg).into();
                        }
                    }
                    Opt::UpperL => {
                        if let FollowLinks::NoFollow = behavior.follow_links {
                            behavior.follow_links = FollowLinks::All;
                        } else {
                            let msg = format!("`{option}` cannot but used with `{}`", Opt::UpperH);
                            return Error::invalid_argument(msg).into();
                        }
                    }
                    // GROUP: Follow links (END)

                    // GROUP: Directory (BEGIN)
                    Opt::LowerD => {
                        if behavior.recursive_dir_walk {
                            let msg = format!("`{option}` cannot be used with `{}`", Opt::UpperR);
                            return Error::invalid_argument(msg).into();
                        } else {
                            behavior.treat_dir_operands_as_regular_files = true;
                        }
                    }
                    Opt::UpperR => behavior.recursive_dir_walk = true,
                    // GROUP: Directory (END)

                    // GROUP: Output Format (BEGIN)
                    Opt::LowerL | Opt::LowerN | Opt::LowerO | Opt::LowerG => {
                        if disable_long {
                            continue;
                        }
                        if let Some(format) = output_format.as_ref()
                            && !LONG_OPTS.contains(format)
                        {
                            let msg = format!("`{option}` cannot be used with `{format}`");
                            return Error::invalid_argument(msg).into();
                        } else if let OutputFormat::Long(long) = &mut behavior.output_format {
                            long.exclude_owner = long.exclude_owner || option == Opt::LowerG;
                            long.exclude_group = long.exclude_group || option == Opt::LowerO;
                            long.owner_group_id = long.owner_group_id || option == Opt::LowerN;
                        } else {
                            behavior.output_format = match option {
                                Opt::LowerL => OutputFormat::Long(Long::default()),
                                Opt::LowerG => OutputFormat::Long(Long {
                                    exclude_owner: true,
                                    ..Default::default()
                                }),
                                Opt::LowerN => OutputFormat::Long(Long {
                                    owner_group_id: true,
                                    ..Default::default()
                                }),
                                Opt::LowerO => OutputFormat::Long(Long {
                                    exclude_group: true,
                                    ..Default::default()
                                }),
                                _ => unreachable!(),
                            };
                            output_format = Some(option);
                        }
                    }
                    Opt::LowerM | Opt::LowerX | Opt::UpperC | Opt::Number1 => {
                        if let Some(format) = output_format
                            && format != option
                        {
                            let msg = format!("`{option}` cannot be used with `{format}`");
                            return Error::invalid_argument(msg).into();
                        } else {
                            behavior.output_format = match option {
                                Opt::LowerM => OutputFormat::CommaSeparated,
                                Opt::LowerX => OutputFormat::MultiColumnHorizontalSort,
                                Opt::UpperC => OutputFormat::MultiColumn,
                                Opt::Number1 => OutputFormat::OneEntryPerLine,
                                _ => unreachable!(),
                            };
                            output_format = Some(option);
                        }
                    } // GROUP: Output Format (END)
                }
            }
        }
    }

    if operands.is_empty() {
        let cwd = current_dir()
            .map(PathBuf::into_os_string)
            .io_error("failed to get current working directory")?;
        operands.push(cwd);
    }

    Ok((operands, behavior))
}

pub fn help_text(binary_name: &str, version: &str) -> String {
    HELP.trim_start()
        .replace("#BIN_NAME", binary_name)
        .replace("#VERSION", version)
}

use crate::{arg::ProgramBehavior, color::Colorizer, file::FileInfo};
use posix_ish_utils::size::{human_bin, human_si};

/// inode, size, long
pub struct Formatter<'a> {
    pub max_entry_physical_width: usize,
    behavior: &'a ProgramBehavior,
    colorizer: &'a Box<dyn Colorizer>,
    format: Format,
    entries: Vec<Entry>,
}

enum Format {
    Column,
    CommaSeparated,
    Long {
        max_ino_physical_width: usize,
        max_size_physical_width: usize,
    },
}

/// Widths take do not take into account ANSI escape sequences. The `ansi_escape_char_count` field
/// is used to indicate how many ANSI escape characters are used.
#[derive(Default)]
pub struct Entry {
    name: String,
    name_physical_width: usize,
    ino: String,
    ino_physical_width: usize,
    size: String,
    size_physical_width: usize,
    ansi_escape_char_count: usize,
}

impl<'a> Formatter<'a> {
    pub fn new_column_layout(
        behavior: &'a ProgramBehavior,
        colorizer: &'a Box<dyn Colorizer>,
        entries: &[FileInfo],
    ) -> Self {
        Self::new(
            behavior,
            colorizer,
            entries,
            Format::Column,
        )
    }

    pub fn new_comma_sep_layout(
        behavior: &'a ProgramBehavior,
        colorizer: &'a Box<dyn Colorizer>,
        entries: &[FileInfo],
    ) -> Self {
        Self::new(behavior, colorizer, entries, Format::CommaSeparated)
    }

    fn new(
        behavior: &'a ProgramBehavior,
        colorizer: &'a Box<dyn Colorizer>,
        entries: &[FileInfo],
        format: Format,
    ) -> Self {
        let mut formatter = Self {
            behavior,
            colorizer,
            entries: Vec::with_capacity(entries.len()),
            format,
            max_entry_physical_width: 0,
        };
        entries.iter().for_each(|ent| formatter.register(ent));

        formatter
    }

    pub fn register(&mut self, info: &FileInfo) {
        let mut entry = Entry::default();

        let (name, name_ansi_char_count) = self.colorizer.name(info);
        entry.ansi_escape_char_count += name_ansi_char_count;
        entry.name_physical_width = name.len() - name_ansi_char_count;
        entry.name = name;

        if self.behavior.include_file_serial_number {
            let ino = format!("{}", info.ino);
            entry.ino_physical_width = ino.len() + 1;
            entry.ino = ino;
        }

        if self.behavior.include_block_size {
            if self.behavior.human_readable_size {
                if self.behavior.si_units {
                    entry.size = format!("{}", human_si(info.blocks, self.behavior.blksize));
                } else {
                    entry.size = format!("{}", human_bin(info.blocks, self.behavior.blksize));
                }
            } else {
                entry.size = format!("{}", info.blocks);
            }
            entry.size_physical_width = entry.size.len() + 1;
        }

        let size_len = entry.size.len();
        let ino_len = entry.ino.len();

        match &mut self.format {
            Format::Long {
                max_ino_physical_width,
                max_size_physical_width,
            } => {
                *max_ino_physical_width = (*max_ino_physical_width).max(ino_len);
                *max_size_physical_width = (*max_size_physical_width).max(size_len);
                self.max_entry_physical_width = self.max_entry_physical_width.max(entry.physical_width());
            }
            Format::Column => {
                // Pad for tailing white space
                self.max_entry_physical_width = self.max_entry_physical_width.max(entry.physical_width() + 1);
            }
            _ => {
                self.max_entry_physical_width = self.max_entry_physical_width.max(entry.physical_width());
            }
        }
        self.entries.push(entry);
    }

    pub fn get_formatted(&self, idx: usize) -> Option<String> {
        let Entry {
            name,
            name_physical_width,
            ino,
            ino_physical_width,
            size,
            size_physical_width,
            ansi_escape_char_count,
        } = &self.entries.get(idx)?;

        let out = match self.format {
            Format::CommaSeparated => {
                format!(
                    "{ino:<iw$}{size:<sw$}{name}",
                    iw = ino_physical_width,
                    sw = size_physical_width
                )
            }
            Format::Column => {
                format!(
                    "{:<mw$}",
                    format!(
                        "{ino:<iw$}{size:<sw$}{name}",
                        iw = ino_physical_width,
                        sw = size_physical_width
                    ),
                    mw = self.max_entry_physical_width + ansi_escape_char_count,
                )
            }
            Format::Long {
                max_ino_physical_width,
                max_size_physical_width,
            } => {
                todo!()
            }
        };

        Some(out)
    }
}

impl Entry {
    /// How many columns it takes to render this entry.
    fn physical_width(&self) -> usize {
        self.ino_physical_width + self.size_physical_width + self.name_physical_width
    }

    /// The actual string length which includes the ansi escape characters
    fn logical_width(&self) -> usize {
        self.ino_physical_width
            + self.size_physical_width
            + self.name_physical_width
            + self.ansi_escape_char_count
    }
}

use crate::{arg::ProgramBehavior, color::Colorizer, file::FileInfo};
use posix_ish_utils::size::{human_bin, human_si};

/// inode, size, long
pub struct Formatter<'a> {
    pub max_entry_physical_width: usize,
    pub max_name_physical_width: usize,
    pub max_ino_physical_width: usize,
    pub max_size_physical_width: usize,
    behavior: &'a ProgramBehavior,
    colorizer: &'a Box<dyn Colorizer>,
    format: Format,
    entries: Vec<Entry>,
}

enum Format {
    Tabular,
    CommaSeparated,
    Long,
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
    pub fn new_tabular_layout(
        behavior: &'a ProgramBehavior,
        colorizer: &'a Box<dyn Colorizer>,
        entries: &[FileInfo],
    ) -> Self {
        Self::new(behavior, colorizer, entries, Format::Tabular)
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
            max_name_physical_width: 0,
            max_ino_physical_width: 0,
            max_size_physical_width: 0,
            max_entry_physical_width: 0,
        };
        entries.iter().for_each(|ent| formatter.register(ent));

        formatter
    }

    pub fn register(&mut self, info: &FileInfo) {
        let mut entry = Entry::default();

        let (name, name_ansi_char_count) = self.colorizer.name(info);
        entry.name = name;
        entry.ansi_escape_char_count += name_ansi_char_count;
        entry.name_physical_width = entry.name.len() - name_ansi_char_count;
        self.max_name_physical_width = self.max_name_physical_width.max(entry.name_physical_width);

        if self.behavior.include_file_serial_number {
            let ino = format!("{} ", info.ino);
            entry.ino = ino;
            entry.ino_physical_width = entry.ino.len();
            self.max_ino_physical_width = self.max_ino_physical_width.max(entry.ino.len());
        }

        if self.behavior.include_block_size {
            if self.behavior.human_readable_size {
                if self.behavior.si_units {
                    entry.size = format!("{} ", human_si(info.blocks, self.behavior.blksize));
                } else {
                    entry.size = format!("{} ", human_bin(info.blocks, self.behavior.blksize));
                }
            } else {
                entry.size = format!("{} ", info.blocks);
            }
            entry.size_physical_width = entry.size.len();
            self.max_size_physical_width = self.max_size_physical_width.max(entry.size.len());
        }

        let current_max = self.max_ino_physical_width
            + self.max_size_physical_width
            + self.max_name_physical_width;

        match &mut self.format {
            Format::Tabular => {
                // Pad for tailing white space
                self.max_entry_physical_width = self.max_entry_physical_width.max(current_max + 1);
            }
            _ => {
                self.max_entry_physical_width = self.max_entry_physical_width.max(current_max);
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
            Format::Tabular => {
                // Note the extra space
                format!(
                    "{ino:>iw$}{size:>sw$}{name:<nw$} ",
                    iw = self.max_ino_physical_width,
                    sw = self.max_size_physical_width,
                    nw = self.max_name_physical_width + ansi_escape_char_count,
                )
            }
            Format::Long => todo!(),
        };

        Some(out)
    }
}

use crate::{
    arg::{Long, ProgramBehavior},
    color::Colorizer,
    fs::file::FileInfo,
    output::time::mtime,
};
use posix_ish_utils::{
    fs::{
        file::{FileType, rwx},
        get_group, get_user,
    },
    size::{apparent_human_bin, apparent_human_si},
};

pub struct Formatter<'a> {
    pub max_entry_physical_width: usize,
    pub max_name_physical_width: usize,
    pub max_ino_physical_width: usize,
    pub max_size_physical_width: usize,
    pub max_nlink_physical_width: usize,
    pub max_owner_physical_width: usize,
    pub max_group_physical_width: usize,
    pub max_len_physical_width: usize,

    pb: &'a ProgramBehavior,
    colorizer: &'a dyn Colorizer,
    format: Format,
    entries: Vec<Entry>,
}

enum Format {
    Tabular,
    CommaSeparated,
    Long(Long),
}

/// How many physical columns in the TTY it takes to render a sequence of text. It does not take
/// into account any ANSI escape sequence characters.
type PhysicalWidth = usize;

/// Widths take do not take into account ANSI escape sequences. The `ansi_escape_char_count` field
/// is used to indicate how many ANSI escape characters are used.
#[derive(Default)]
pub struct Entry {
    name: (String, PhysicalWidth),
    ino: (String, PhysicalWidth),
    size: (String, PhysicalWidth),
    referent: String,
    owner: String,
    group: String,
    nlinks: String,
    len: String,
    // Mode column is fixed width in terminal
    mode: String,
    // mtime column is fixed width in terminal
    mtime: String,
    ansi_escape_char_count: usize,
}

impl<'a> Formatter<'a> {
    pub fn new_tabular_layout(
        pb: &'a ProgramBehavior,
        colorizer: &'a dyn Colorizer,
        entries: &[FileInfo],
    ) -> Self {
        Self::new(pb, colorizer, entries, Format::Tabular)
    }

    pub fn new_comma_sep_layout(
        pb: &'a ProgramBehavior,
        colorizer: &'a dyn Colorizer,
        entries: &[FileInfo],
    ) -> Self {
        Self::new(pb, colorizer, entries, Format::CommaSeparated)
    }

    pub fn new_long_layout(
        pb: &'a ProgramBehavior,
        colorizer: &'a dyn Colorizer,
        entries: &[FileInfo],
        opt: Long,
    ) -> Self {
        Self::new(pb, colorizer, entries, Format::Long(opt))
    }

    fn new(
        pb: &'a ProgramBehavior,
        colorizer: &'a dyn Colorizer,
        entries: &[FileInfo],
        format: Format,
    ) -> Self {
        let mut formatter = Self {
            pb,
            colorizer,
            entries: Vec::with_capacity(entries.len()),
            format,
            max_name_physical_width: 0,
            max_ino_physical_width: 0,
            max_size_physical_width: 0,
            max_entry_physical_width: 0,
            max_nlink_physical_width: 0,
            max_owner_physical_width: 0,
            max_group_physical_width: 0,
            max_len_physical_width: 0,
        };
        entries.iter().for_each(|ent| formatter.register(ent));

        formatter
    }

    pub fn register(&mut self, info: &FileInfo) {
        let mut entry = Entry::default();

        let (mut name, name_ansi_char_count) = self.colorizer.name(info);

        if self.pb.append_fslash_to_dir || self.pb.include_file_type_symbol {
            match info.file_type {
                FileType::Dir => name.push('/'),
                FileType::Fifo => name.push('|'),
                FileType::SymLink => name.push('@'),
                _ if info.is_executable() => name.push('*'),
                _ => (),
            }
        }
        let name_physical_width = name.len() - name_ansi_char_count;
        entry.name = (name, name_physical_width);
        entry.ansi_escape_char_count += name_ansi_char_count;
        self.max_name_physical_width = self.max_name_physical_width.max(name_physical_width);

        if let Some(referent) = info.referent.as_ref() {
            entry.referent = format!(" -> {}", referent.display());
        }

        if self.pb.include_file_serial_number {
            let ino = format!("{} ", info.ino);
            let ino_physical_width = ino.len();
            entry.ino = (ino, ino_physical_width);
            self.max_ino_physical_width = self.max_ino_physical_width.max(ino_physical_width);
        }

        if self.pb.include_block_size {
            let size = format!("{} ", info.blocks);
            let size_physical_width = size.len();
            entry.size = (size, size_physical_width);
            self.max_size_physical_width = self.max_size_physical_width.max(size_physical_width);
        }

        let current_max = self.max_ino_physical_width
            + self.max_size_physical_width
            + self.max_name_physical_width;

        match &self.format {
            Format::Long(opt) => {
                // Mode column is fixed width in terminal
                entry.mode = rwx(&info.file_type, info.mode);

                let links = format!("{}", info.nlink);
                let link_len = links.len();
                entry.nlinks = links;
                self.max_nlink_physical_width = self.max_nlink_physical_width.max(link_len);

                if opt.owner_group_id {
                    let o = format!("{}", info.uid);
                    let o_len = o.len();
                    entry.owner = o;
                    self.max_owner_physical_width = self.max_owner_physical_width.max(o_len);

                    let g = format!("{}", info.gid);
                    let g_len = g.len();
                    entry.group = g;
                    self.max_group_physical_width = self.max_group_physical_width.max(g_len);
                } else {
                    let o = get_user(info.uid).unwrap_or_else(|_| String::from("-"));
                    let o_len = o.len();
                    entry.owner = o;
                    self.max_owner_physical_width = self.max_owner_physical_width.max(o_len);

                    let g = get_group(info.gid).unwrap_or_else(|_| String::from("_"));
                    let g_len = g.len();
                    entry.group = g;
                    self.max_group_physical_width = self.max_group_physical_width.max(g_len);
                }

                let len = if !self.pb.human_readable_size {
                    format!("{}", info.len)
                } else if self.pb.si_units {
                    format!("{}", apparent_human_si(info.len).display(true))
                } else {
                    format!("{}", apparent_human_bin(info.len).display(true))
                };
                let len_len = len.len();
                entry.len = len;
                self.max_len_physical_width = self.max_len_physical_width.max(len_len);

                // mtime column is fixed width in terminal
                entry.mtime = mtime(info.mtime, self.pb.program_start);
            }
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
            mode,
            nlinks,
            owner,
            group,
            mtime,
            len,
            referent,
            name: (name, _),
            ino: (ino, ino_physical_width),
            size: (size, size_physical_width),
            ansi_escape_char_count,
            ..
        } = &self.entries.get(idx)?;

        let out = match &self.format {
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
            // <mode> <nlink> <owner> <group> <apparent size> <mtime> <name>
            Format::Long(opt) => {
                if opt.exclude_owner {
                    format!(
                        "{ino:>iw$}{size:>sw$}{mode} {nlinks:>lw$} {group:>gw$} {len:>lenw$} {mtime} {name}{referent}",
                        iw = self.max_ino_physical_width,
                        sw = self.max_size_physical_width,
                        lw = self.max_nlink_physical_width,
                        gw = self.max_group_physical_width,
                        lenw = self.max_len_physical_width,
                    )
                } else if opt.exclude_group {
                    format!(
                        "{ino:>iw$}{size:>sw$}{mode} {nlinks:>lw$} {owner:>ow$} {len:>lenw$} {mtime} {name}{referent}",
                        iw = self.max_ino_physical_width,
                        sw = self.max_size_physical_width,
                        lw = self.max_nlink_physical_width,
                        ow = self.max_owner_physical_width,
                        lenw = self.max_len_physical_width,
                    )
                } else {
                    format!(
                        "{ino:>iw$}{size:>sw$}{mode} {nlinks:>lw$} {owner:>ow$} {group:>gw$} {len:>lenw$} {mtime} {name}{referent}",
                        iw = self.max_ino_physical_width,
                        sw = self.max_size_physical_width,
                        lw = self.max_nlink_physical_width,
                        ow = self.max_owner_physical_width,
                        gw = self.max_group_physical_width,
                        lenw = self.max_len_physical_width,
                    )
                }
            }
        };

        Some(out)
    }
}

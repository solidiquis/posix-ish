use crate::{
    arg::{IncludeAll, Long, OutputFormat, ProgramBehavior, Sort},
    color::{Colorizer, init_colorizer},
    file::FileInfo,
};
use posix_ish_utils::error::Result;

/// Sorts, filters, and prints.
pub fn print(mut entries: Vec<FileInfo>, pb: &ProgramBehavior) -> Result<()> {
    let colorizer = init_colorizer()?;

    // Filter
    entries.retain_mut(|dent| matches!(pb.include_all, IncludeAll::ExcludeHidden) && !dent.hidden);

    // Sort
    match pb.sort {
        Sort::Size if pb.reverse_sort => {
            entries.sort_unstable_by(|a, b| b.blocks.cmp(&a.blocks).then(b.name.cmp(&a.name)));
        }
        Sort::Size => {
            entries.sort_unstable_by(|a, b| a.blocks.cmp(&b.blocks).then(a.name.cmp(&b.name)));
        }
        Sort::Mod if pb.reverse_sort => {
            entries.sort_unstable_by(|a, b| b.mtime.cmp(&a.mtime).then(b.name.cmp(&a.name)));
        }
        Sort::Mod => {
            entries.sort_unstable_by(|a, b| a.mtime.cmp(&b.mtime).then(a.name.cmp(&b.name)));
        }
        Sort::Access if pb.reverse_sort => {
            entries.sort_unstable_by(|a, b| b.atime.cmp(&a.atime).then(b.name.cmp(&a.name)));
        }
        Sort::Access => {
            entries.sort_unstable_by(|a, b| a.atime.cmp(&b.atime).then(a.name.cmp(&b.name)));
        }
        Sort::Status if pb.reverse_sort => {
            entries.sort_unstable_by(|a, b| b.ctime.cmp(&a.atime).then(b.name.cmp(&a.name)));
        }
        Sort::Status => {
            entries.sort_unstable_by(|a, b| a.ctime.cmp(&b.atime).then(a.name.cmp(&b.name)));
        }
        Sort::Alphabetical if pb.reverse_sort => {
            entries.sort_unstable_by(|a, b| b.name.cmp(&a.name));
        }
        Sort::Alphabetical => {
            entries.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        }
        Sort::None => entries.reverse(),
    }

    match &pb.output_format {
        OutputFormat::CommaSeparated => comma_separated(entries, colorizer),
        OutputFormat::Long(opts) => long(entries, pb, opts),
        OutputFormat::MultiColumnHorizontalSort => multi_column_horizontal_sort(entries, colorizer),
        OutputFormat::MultiColumn => multi_column(entries, colorizer),
        OutputFormat::OneEntryPerLine => one_entry_per_line(entries, colorizer),
    }
}

/// `-m`
fn comma_separated(entries: Vec<FileInfo>, colorizer: Box<dyn Colorizer>) -> Result<()> {
    let mut out = String::new();
    for (i, entry) in entries.iter().enumerate() {
        let (formatted, _) = colorizer.apply(entry);
        out.push_str(&format!("{formatted}"));

        if i < entries.len() - 1 {
            out.push_str(", ");
        }
    }
    println!("{out}");
    Ok(())
}

/// `-l`
fn long(entries: Vec<FileInfo>, pb: &ProgramBehavior, opts: &Long) -> Result<()> {
    todo!()
}

/// `-x`
fn multi_column_horizontal_sort(
    entries: Vec<FileInfo>,
    colorizer: Box<dyn Colorizer>,
) -> Result<()> {
    let (num_cols, num_rows, col_width) = match util::get_table_props(&entries) {
        Ok(Some((c, r, w))) => (c, r, w),
        Ok(None) => return one_entry_per_line(entries, colorizer),
        Err(err) => return Err(err),
    };

    let mut out = String::new();

    for (i, chunk) in entries.chunks(num_cols).enumerate() {
        for entry in chunk {
            let (colorized, ansi_chars_len) = colorizer.apply(&entry);
            let formatted = format!("{colorized:<width$}", width = col_width + ansi_chars_len);
            out.push_str(&formatted);
        }
        if i < num_rows - 1 {
            out.push('\n');
        }
    }
    println!("{out}");
    Ok(())
}

/// `-C` i.e. default
fn multi_column(entries: Vec<FileInfo>, colorizer: Box<dyn Colorizer>) -> Result<()> {
    let (_, num_rows, col_width) = match util::get_table_props(&entries) {
        Ok(Some((c, r, w))) => (c, r, w),
        Ok(None) => return one_entry_per_line(entries, colorizer),
        Err(err) => return Err(err),
    };

    let mut out = String::new();

    for offset in 0..num_rows {
        let mut cursor = offset;

        while cursor < entries.len() {
            let (colorized, ansi_chars_len) = colorizer.apply(&entries[cursor]);
            let formatted = format!("{colorized:<width$}", width = col_width + ansi_chars_len);
            out.push_str(&formatted);
            cursor += num_rows;
        }
        if offset < num_rows - 1 {
            out.push('\n');
        }
    }
    println!("{out}");
    Ok(())
}

/// `-1`
fn one_entry_per_line(entries: Vec<FileInfo>, colorizer: Box<dyn Colorizer>) -> Result<()> {
    let mut out = String::new();
    for (i, entry) in entries.iter().enumerate() {
        let (formatted, _) = colorizer.apply(entry);
        out.push_str(&format!("{formatted}"));

        if i < entries.len() - 1 {
            out.push('\n');
        }
    }
    println!("{out}");
    Ok(())
}

mod util {
    use crate::file::FileInfo;
    use posix_ish_utils::error::Result;

    /// Returns window column count, row count, and column width.
    pub fn get_table_props(entries: &[FileInfo]) -> Result<Option<(usize, usize, usize)>> {
        let col_width = entries.iter().fold(0, |max, dent| max.max(dent.name.len())) + 1;
        let libc::winsize { ws_col, .. } = posix_ish_utils::tty::get_winsize()?;

        let num_columns = match u16::try_from(col_width) {
            Ok(width) if width > 0 && width < ws_col => usize::from(ws_col / width),
            _ => return Ok(None),
        };

        let num_rows = entries.len().div_ceil(usize::from(num_columns));

        Ok(Some((num_columns, num_rows, col_width)))
    }
}

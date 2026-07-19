use crate::{
    arg::{IncludeAll, Long, OutputFormat, ProgramBehavior, Sort},
    color::{Colorizer, init_colorizer},
    file::FileInfo,
};
use posix_ish_utils::{error::Result, tty};

/// Concerned with output format of a file.
mod file;
use file::Formatter;

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

    let out = match &pb.output_format {
        OutputFormat::CommaSeparated => comma_separated(&entries, &pb, &colorizer)?,
        OutputFormat::Long(opts) => long(&entries, opts, &colorizer)?,
        OutputFormat::TableRowMajor => tabular(&entries, &pb, &colorizer, false)?,
        OutputFormat::TableColumnMajor => tabular(&entries, &pb, &colorizer, true)?,
        OutputFormat::OneEntryPerLine => one_entry_per_line(&entries, &colorizer)?,
    };

    println!("{out}");

    Ok(())
}

/// `-l`
fn long(entries: &[FileInfo], opts: &Long, colorizer: &Box<dyn Colorizer>) -> Result<String> {
    todo!()
}

/// `-m`
fn comma_separated(
    entries: &[FileInfo],
    pb: &ProgramBehavior,
    colorizer: &Box<dyn Colorizer>,
) -> Result<String> {
    let formatter = Formatter::new_comma_sep_layout(pb, colorizer, &entries);

    let mut out = String::new();
    for i in 0..entries.len() {
        let Some(formatted) = formatter.get_formatted(i) else {
            break;
        };
        out.push_str(&format!("{formatted}"));

        if i < entries.len() - 1 {
            out.push_str(", ");
        }
    }
    Ok(out)
}

/// `row_major` set to `true` corresponds with `-x` otherwise `-C`. 
fn tabular(
    entries: &[FileInfo],
    pb: &ProgramBehavior,
    colorizer: &Box<dyn Colorizer>,
    row_major: bool,
) -> Result<String> {
    let formatter = Formatter::new_column_layout(pb, colorizer, &entries);
    let win_width = tty::get_winsize().map(|win| usize::from(win.ws_col))?;
    let max_col_width = formatter.max_entry_physical_width;

    if max_col_width >= win_width {
        return one_entry_per_line(entries, colorizer);
    }

    let num_col = win_width / max_col_width;
    let num_rows = entries.len().div_ceil(num_col);

    let mut out = String::new();

    if row_major {
        for offset in 0..num_rows {
            let mut cursor = offset;

            while cursor < entries.len() {
                let Some(formatted) = formatter.get_formatted(cursor) else {
                    break;
                };
                out.push_str(&formatted);
                cursor += num_rows;
            }
            if offset < num_rows - 1 {
                out.push('\n');
            }
        }
    } else {
        for i in 0..num_rows {
            let row_offset = num_col * i;
            for j in 0..num_col {
                let idx = row_offset + j;
                let Some(formatted) = formatter.get_formatted(idx) else {
                    break;
                };
                out.push_str(&formatted);
            }
            if i < num_rows - 1 {
                out.push('\n');
            }
        }
    }

    Ok(out)
}

/// `-1`
fn one_entry_per_line(entries: &[FileInfo], colorizer: &Box<dyn Colorizer>) -> Result<String> {
    let mut out = String::new();
    for (i, entry) in entries.iter().enumerate() {
        let (formatted, _) = colorizer.name(entry);
        out.push_str(&format!("{formatted}"));

        if i < entries.len() - 1 {
            out.push('\n');
        }
    }
    Ok(out)
}

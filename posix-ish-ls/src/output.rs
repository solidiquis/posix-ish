use crate::{
    arg::{IncludeAll, Long, OutputFormat, ProgramBehavior, Sort},
    color::{Colorizer, init_colorizer},
    error::Result,
    file::FileInfo,
};

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
        Sort::None => entries.reverse(),
    }

    match &pb.output_format {
        OutputFormat::CommaSeparated => comma_separated(entries, colorizer),
        OutputFormat::Long(opts) => long(entries, pb, opts),
        OutputFormat::MultiColumnHorizontalSort => multi_column_horizontal_sort(entries, pb),
        OutputFormat::MultiColumn => multi_column(entries, pb),
        OutputFormat::OneEntryPerLine => one_entry_per_line(entries, pb),
    }
}

fn comma_separated(entries: Vec<FileInfo>, colorizer: Box<dyn Colorizer>) -> Result<()> {
    let mut out = String::new();
    for entry in &entries[..entries.len() - 1] {
        let formatted = colorizer.apply(entry);
        out.push_str(&formatted);
        out.push_str(", ");
    }
    out.push_str(&colorizer.apply(&entries[entries.len() - 1]));
    println!("{out}");
    Ok(())
}

fn long(entries: Vec<FileInfo>, pb: &ProgramBehavior, opts: &Long) -> Result<()> {
    todo!()
}

fn multi_column_horizontal_sort(entries: Vec<FileInfo>, pb: &ProgramBehavior) -> Result<()> {
    todo!()
}

fn multi_column(entries: Vec<FileInfo>, pb: &ProgramBehavior) -> Result<()> {
    todo!()
}

fn one_entry_per_line(entries: Vec<FileInfo>, pb: &ProgramBehavior) -> Result<()> {
    todo!()
}

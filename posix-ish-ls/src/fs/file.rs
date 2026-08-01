use crate::arg::{FollowLinks, OutputFormat, ProgramBehavior, Sort};
use posix_ish_utils::{
    error::{Error, Result, ToPosixishResult},
    fs::file::FileType,
};
use std::{
    ffi::OsStr,
    fs::{self, DirEntry, Metadata},
    os::unix::fs::MetadataExt,
    path::PathBuf,
};

#[derive(Default)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub file_type: FileType,
    pub hidden: bool,
    pub referent: Option<PathBuf>,
    pub ino: u64,
    pub nlink: u64,
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub blocks: u64,
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
    pub len: u64,
}

impl FileInfo {
    pub fn try_new(entry: DirEntry, pb: &ProgramBehavior) -> Result<Self> {
        let name = Self::sanitize_file_name(&entry.file_name(), pb.non_printable_and_tabs_to_qmark);
        let path = entry.path();
        let file_type = entry.file_type().map_or(FileType::Unknown, FileType::from);
        let hidden = name.starts_with('.');

        let referent = if matches!(file_type, FileType::SymLink) {
            let target = entry
                .path()
                .canonicalize()
                .io_error("failed to resolve link")?;
            Some(target)
        } else {
            None
        };

        if !Self::need_metadata(pb) {
            return Ok(Self {
                name,
                path,
                hidden,
                referent,
                file_type,
                ..Default::default()
            });
        }
        let metadata = match pb.follow_links {
            FollowLinks::NoFollow => entry.metadata(),
            _ => {
                if let Some(target) = &referent {
                    fs::metadata(target)
                } else {
                    entry.metadata()
                }
            }
        }
        .io_error("failed to read file metadata")?;

        let ino = metadata.ino();
        let uid = metadata.uid();
        let gid = metadata.gid();
        let mode = metadata.mode();
        let nlink = metadata.nlink();
        let blocks = metadata.blocks();
        let atime = metadata.atime();
        let mtime = metadata.mtime();
        let ctime = metadata.ctime();
        let len = metadata.len();

        Ok(Self {
            name,
            path,
            file_type,
            hidden,
            referent,
            ino,
            nlink,
            uid,
            gid,
            mode,
            blocks,
            atime,
            mtime,
            ctime,
            len,
        })
    }

    fn need_metadata(pb: &ProgramBehavior) -> bool {
        pb.sort == Sort::Size
            || pb.follow_links != FollowLinks::NoFollow
            || pb.include_block_size
            || matches!(pb.output_format, OutputFormat::Long(_))
    }

    /// Does user, group, or other have executable permissions over this file.
    pub fn is_executable(&self) -> bool {
        self.mode & 0o111 > 0
    }

    /// Replace non-printable characters with '?' if `replace_non_printables` is `true`. Non-unicode
    /// sequences will always be replaced with the unicode decode error replacement character.
    fn sanitize_file_name(name: &OsStr, replace_non_printables: bool) -> String {
        if replace_non_printables {
            name.to_string_lossy()
                .into_owned()
                .chars()
                .map(|c| if c.is_control() || c == '\t' { '?' } else { c })
                .collect()
        } else {
            name.to_string_lossy().into_owned()
        }
    }
}

impl TryFrom<(&OsStr, &Metadata, &ProgramBehavior)> for FileInfo {
    type Error = Error;

    fn try_from((path, md, pb): (&OsStr, &Metadata, &ProgramBehavior)) -> Result<Self> {
        let path = PathBuf::from(path);
        let name = path
            .canonicalize()
            .io_error("failed to canonicalize provided path")?
            .file_name()
            .map_or_else(
                || format!("{}", path.display()),
                |s| Self::sanitize_file_name(s, pb.non_printable_and_tabs_to_qmark),
            );
        let hidden = name.starts_with('.');
        let file_type = FileType::from(md.file_type());
        let ino = md.ino();
        let uid = md.uid();
        let gid = md.gid();
        let mode = md.mode();
        let nlink = md.nlink();
        let blocks = md.blocks();
        let atime = md.atime();
        let mtime = md.mtime();
        let ctime = md.ctime();
        let len = md.len();

        let referent = if matches!(file_type, FileType::SymLink) {
            let target = path.canonicalize().io_error("failed to resolve link")?;

            Some(target)
        } else {
            None
        };

        Ok(Self {
            name,
            path,
            file_type,
            hidden,
            referent,
            ino,
            nlink,
            uid,
            gid,
            mode,
            blocks,
            atime,
            mtime,
            ctime,
            len,
        })
    }
}

#[test]
fn test_sanitize_file_name() {
    let danger = std::ffi::OsString::from("\x1b[31;43mDANGER_FILE\x1b[0m");
    let sanitized = FileInfo::sanitize_file_name(&danger, true);
    let expected = "?[31;43mDANGER_FILE?[0m";
    assert_eq!(expected, sanitized);
}

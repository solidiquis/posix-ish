use crate::arg::{FollowLinks, OutputFormat, ProgramBehavior, Sort};
use posix_ish_utils::error::{Error, Result, ToLsResult};
use std::{
    ffi::OsStr,
    fs::{self, DirEntry, Metadata},
    os::unix::fs::{FileTypeExt, MetadataExt},
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
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum FileType {
    File,
    Dir,
    SymLink,
    BlockDev,
    CharDev,
    Fifo,
    Socket,
    #[default]
    Unknown,
}

impl FileInfo {
    pub fn try_new(entry: DirEntry, behavior: &ProgramBehavior) -> Result<Self> {
        let name = entry.file_name().to_string_lossy().to_string();
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

        if !Self::need_metadata(behavior) {
            return Ok(Self {
                name,
                path,
                hidden,
                referent,
                file_type,
                ..Default::default()
            });
        }
        let metadata = match behavior.follow_links {
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
        })
    }

    fn need_metadata(behavior: &ProgramBehavior) -> bool {
        behavior.sort == Sort::Size
            || behavior.follow_links != FollowLinks::NoFollow
            || behavior.include_block_size
            || matches!(behavior.output_format, OutputFormat::Long(_))
    }

    /// Does user, group, or other have executable permissions over this file.
    pub fn is_executable(&self) -> bool {
        self.mode & 0o111 > 0
    }
}

impl TryFrom<(&OsStr, &Metadata)> for FileInfo {
    type Error = Error;

    fn try_from((path, md): (&OsStr, &Metadata)) -> Result<Self> {
        let path = PathBuf::from(path);
        let name = path
            .canonicalize()
            .io_error("failed to canonicalize provided path")?
            .file_name()
            .map_or_else(
                || format!("{}", path.display()),
                |s| s.to_os_string().to_string_lossy().to_string(),
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
        })
    }
}

impl From<std::fs::FileType> for FileType {
    fn from(ft: std::fs::FileType) -> Self {
        if ft.is_file() {
            Self::File
        } else if ft.is_dir() {
            Self::Dir
        } else if ft.is_symlink() {
            Self::SymLink
        } else if ft.is_block_device() {
            Self::BlockDev
        } else if ft.is_char_device() {
            Self::CharDev
        } else if ft.is_fifo() {
            Self::Fifo
        } else if ft.is_socket() {
            Self::Socket
        } else {
            Self::Unknown
        }
    }
}

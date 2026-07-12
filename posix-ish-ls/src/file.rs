use std::{
    ffi::{OsStr, OsString},
    fs::{self, DirEntry, Metadata},
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::PathBuf,
};

use crate::{
    arg::{FollowLinks, OutputFormat, ProgramBehavior, Sort},
    error::{Error, Result, ToLsResult},
};

/// 512B
const BLKSIZE: u64 = 512;

#[derive(Default)]
pub struct FileInfo {
    pub name: OsString,
    pub path: PathBuf,
    pub file_type: FileType,
    pub hidden: bool,
    pub referent: Option<OsString>,
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

#[derive(Default, PartialEq, Eq)]
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
        let name = entry.file_name();
        let path = entry.path();
        let file_type = entry.file_type().map_or(FileType::Unknown, FileType::from);
        let hidden = name.to_string_lossy().starts_with('.');

        if !Self::need_metadata(behavior) {
            return Ok(Self {
                name,
                path,
                hidden,
                file_type,
                ..Default::default()
            });
        }

        let mut referent = None;
        let metadata = match behavior.follow_links {
            FollowLinks::NoFollow => entry.metadata(),
            _ if file_type != FileType::SymLink => entry.metadata(),
            _ => {
                let target = entry
                    .path()
                    .canonicalize()
                    .io_error("failed to resolve link")?;

                referent = target.file_name().map(|s| s.to_os_string());

                fs::metadata(target)
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
            .file_name()
            .map(|s| s.to_os_string())
            .io_error("expect valid file name")?;
        let hidden = name.to_string_lossy().starts_with('.');
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
        let referent = None;

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

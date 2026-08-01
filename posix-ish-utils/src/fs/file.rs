use std::os::unix::fs::FileTypeExt;

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

/// See [STDOUT](https://pubs.opengroup.org/onlinepubs/009695099/utilities/ls.html) section:
///
/// `<entry type>, <owner perms>, <group perms>, <other perms>, <optional alt access flag>`
///
/// File types:
/// - `d`: Directory
/// - `b`: Block device
/// - `c`: Character device
/// - `l`: Link
/// - `p`: Fifo
/// - `s`: Socket
/// - `-`: Regular File
pub fn rwx(ft: &FileType, mode: u32) -> String {
    let mut out = String::with_capacity(10);

    match ft {
        FileType::Dir => out.push('d'),
        FileType::BlockDev => out.push('b'),
        FileType::CharDev => out.push('c'),
        FileType::SymLink => out.push('l'),
        FileType::Fifo => out.push('p'),
        FileType::Socket => out.push('s'),
        _ => out.push('-'),
    };

    for shift in [6, 3, 0] {
        let perm = mode >> shift;
        let r = if perm & 0o4 != 0 { 'r' } else { '-' };
        let w = if perm & 0o2 != 0 { 'w' } else { '-' };
        let x = if perm & 0o1 != 0 { 'x' } else { '-' };
        out.push(r);
        out.push(w);
        out.push(x);
    }

    out
}

#[test]
fn test_rwx() {
    assert_eq!(rwx(&FileType::File, 0o777), String::from("-rwxrwxrwx"));
    assert_eq!(rwx(&FileType::Dir, 0o456), String::from("dr--r-xrw-"));
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

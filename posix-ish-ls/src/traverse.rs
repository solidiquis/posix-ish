use crate::{
    arg::{FollowLinks, ProgramBehavior},
    file::{FileInfo, FileType},
};
use posix_ish_utils::error::{Result, ToLsResult};
use std::{
    collections::{HashMap, HashSet},
    ffi::{OsStr, OsString},
    fs::{self, ReadDir},
};

struct DirTraverseState {
    path: OsString,
    reader: ReadDir,
}

/// Traverse `root` and gather all the relevant file information based the [ProgramBehavior].
/// This function will not sort nor filter - we leave that to be handled downstream.
pub fn traverse(root: &OsStr, pb: &ProgramBehavior) -> Result<Vec<FileInfo>> {
    let root_md = if pb.follow_links == FollowLinks::NoFollow {
        fs::symlink_metadata(root)
    } else {
        fs::metadata(root)
    }
    .io_error("failed to query operand metadata")?;

    let root_info = {
        let mut root = FileInfo::try_from((root, &root_md))?;
        root.name = ".".to_string();
        root.hidden = true;
        root
    };

    if !root_md.is_dir() {
        return Ok(vec![root_info]);
    }

    let readdir = fs::read_dir(root)
        .map(|reader| DirTraverseState {
            reader,
            path: root.to_os_string(),
        })
        .io_error("failed to read directory")?;

    // Directory reader and the dir's corresponding index into entries
    let mut dir_stack = vec![(0, readdir)];
    let mut entries = vec![root_info];

    let mut dir_blocks_map = HashMap::<OsString, u64>::from_iter([(root.to_os_string(), 0)]);

    // Prevent double counting files that refer to the same inode
    let mut accounted_inodes = HashSet::new();

    // Depth first search directory traversal
    while !dir_stack.is_empty() {
        let Some((dir_cursor, current_dir)) = dir_stack.last_mut() else {
            break;
        };

        // No more entries. Pop current dir off stack and assign its size.
        let Some(entry_res) = current_dir.reader.next() else {
            entries[*dir_cursor].blocks = dir_blocks_map
                .get(&current_dir.path)
                .cloned()
                .unwrap_or_default();

            dir_stack.pop();

            continue;
        };

        let entry = entry_res
            .io_error("failed to read an entry")
            .and_then(|dirent| FileInfo::try_new(dirent, pb))?;

        // Track directory size and avoid double counting inodes
        if let Some(blocks) = dir_blocks_map.get_mut(&current_dir.path) {
            if !accounted_inodes.contains(&entry.ino) {
                accounted_inodes.insert(entry.ino);
                *blocks += entry.blocks;
            }
        } else {
            dir_blocks_map.insert(current_dir.path.clone(), 0);
        }

        // Push child directory on top of directory stack
        if entry.file_type == FileType::Dir && pb.recursive_dir_walk {
            let readdir = fs::read_dir(&entry.path).io_error("failed to read directory")?;

            let next_dir = DirTraverseState {
                path: entry.path.as_os_str().to_os_string(),
                reader: readdir,
            };
            dir_stack.push((entries.len(), next_dir));
        }

        entries.push(entry);
    }

    Ok(entries)
}

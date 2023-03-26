use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

pub fn path_writable(path: &std::path::Path) -> super::PathWritability {
    std::fs::metadata(path)
        .map(|stat| {
            // XXX there really has to be a better option here
            let euid = users::get_effective_uid();
            let egid = users::get_effective_gid();
            let file_uid = stat.uid();
            let file_gid = stat.gid();
            let file_mode = stat.permissions().mode();

            if euid == 0
                || ((file_uid == euid) && (file_mode & 0o200 != 0))
                || ((file_gid == egid) && (file_mode & 0o020 != 0))
                || (file_mode & 0o002 != 0)
            {
                super::PathWritability::Writable
            } else {
                super::PathWritability::NotWritable
            }
        })
        .unwrap_or(super::PathWritability::NotExist)
}

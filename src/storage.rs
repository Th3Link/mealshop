use std::{fs, io, path::Path};

/// Publish a new immutable file while the caller holds its store's exclusive writer lock.
pub(crate) fn publish_new(pending: &Path, target: &Path) -> io::Result<()> {
    #[cfg(target_os = "android")]
    {
        match fs::symlink_metadata(target) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Immutable target exists",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        // Android app SELinux policy prohibits hard links. Both names are in the same private
        // directory; device lifetime ownership and the store mutex exclude competing writers.
        fs::rename(pending, target)
    }
    #[cfg(not(target_os = "android"))]
    fs::hard_link(pending, target)
}

pub(crate) fn finish_publish(pending: &Path) -> io::Result<()> {
    #[cfg(target_os = "android")]
    {
        match fs::remove_file(pending) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }
    #[cfg(not(target_os = "android"))]
    fs::remove_file(pending)
}

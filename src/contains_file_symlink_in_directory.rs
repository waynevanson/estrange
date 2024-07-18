use std::{io, path::Path};

pub trait ContainsDirectory {
    fn contains_file_symlink_in_directory(&self) -> Result<bool, io::Error>;
}

impl<T> ContainsDirectory for T
where
    Self: AsRef<Path>,
{
    fn contains_file_symlink_in_directory(&self) -> Result<bool, io::Error> {
        for dir_entry in self.as_ref().read_dir()? {
            let file_type = dir_entry?.file_type()?;

            if file_type.is_file() || file_type.is_symlink() {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

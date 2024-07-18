use std::{
    io,
    path::{Path, PathBuf},
};

pub trait RelativePath {
    fn relative<P>(&self, to: P) -> Result<Self, io::Error>
    where
        Self: Sized,
        P: AsRef<Path>;
}

impl RelativePath for PathBuf {
    // todo: use self as a reference
    fn relative<P>(&self, to: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        if !self.is_absolute() {
            return Err(io::ErrorKind::Other.into());
        }

        let to = to.as_ref();

        if !to.is_absolute() {
            return Err(io::ErrorKind::Other.into());
        }

        let mut left = self.components();
        let mut right = to.components();

        let mut buffer = PathBuf::new();

        let mut l = left.next();
        let mut r = right.next();
        let mut dotted = false;

        loop {
            match (l, r) {
                (Some(from), Some(to)) => {
                    if from != to {
                        buffer.push("..");
                        dotted = true;
                        r = Some(to);
                    } else {
                        r = right.next();
                    }

                    l = left.next();
                }
                (Some(_), None) => {
                    buffer.push("..");
                    dotted = true;
                    l = left.next();
                }
                (None, Some(to)) => {
                    if !dotted {
                        buffer.push(".");
                        dotted = true;
                    }
                    buffer.push(to);
                    r = right.next();
                }
                _ => {
                    if !dotted {
                        buffer.push(".");
                    }
                    break;
                }
            }
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn relative_path_to_deeper() {
        let from = PathBuf::from("/hello/world/say/goodbye");
        let to = PathBuf::from("/hello/world/tell/the/world");

        let result = from.relative(to).unwrap();

        assert_eq!(result, PathBuf::from("../../tell/the/world"))
    }

    #[test]
    fn relative_path_to_shallower() {
        let from = PathBuf::from("/hello/world/say/goodbye");
        let to = PathBuf::from("/hello/world/tell");

        let result = from.relative(to).unwrap();

        assert_eq!(result, PathBuf::from("../../tell"))
    }

    #[test]
    fn relative_path_not_deep() {
        let from = PathBuf::from("/hello/world/say/goodbye");
        let to = PathBuf::from("/hello/world/");

        let result = from.relative(to).unwrap();

        assert_eq!(result, PathBuf::from("../.."))
    }

    #[test]
    fn relative_path_down() {
        let from = PathBuf::from("/hello/");
        let to = PathBuf::from("/hello/world/bro/sup");

        let result = from.relative(to).unwrap();

        assert_eq!(result, PathBuf::from("./world/bro/sup"))
    }

    #[test]
    fn relative_path_same() {
        let from = PathBuf::from("/hello/world/bro/sup");
        let to = PathBuf::from("/hello/world/bro/sup");

        let result = from.relative(to).unwrap();

        assert_eq!(result, PathBuf::from("."))
    }
}

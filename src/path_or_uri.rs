use std::{fmt::Display, path::PathBuf, str::FromStr};

use crate::{Error, Result};
use path_clean::PathClean;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(variant_size_differences)]
/// A local file path or a remote URI.
pub enum PathOrURI {
    /// A local file path.
    Path(PathBuf),
    /// A URI of some kind.
    URI(Url),
}

impl PathOrURI {
    /// Returns whether the contained value is a path.
    #[must_use]
    pub fn is_path(&self) -> bool {
        matches!(self, PathOrURI::Path(_))
    }

    /// Returns whether the contained value is a URI.
    ///
    /// When created from a URI using [`From`], a `file://` URI will be converted to a [`PathBuf`]
    /// and this method will return `false`.
    #[must_use]
    pub fn is_uri(&self) -> bool {
        matches!(self, PathOrURI::URI(_))
    }

    /// Returns the contained value as a URI.
    ///
    /// # Errors
    ///
    /// - The cleaned path does not resolve to an absolute path.
    /// - If the contained path is relative and the current directory does not exist or cannot be
    /// accessed (i.e. [`std::env::current_dir`] fails).
    pub fn uri(&self) -> Result<Url> {
        match self {
            Self::URI(url) => Ok(url.clone()),
            Self::Path(path) => {
                let new_path = std::env::current_dir()?.join(path).clean();
                Url::from_file_path(new_path).map_err(|_| Error::FileToURI(path.clone()))
            }
        }
    }
}

impl FromStr for PathOrURI {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Url>() {
            Ok(url) => Ok(Self::from(url)),
            Err(_) => Ok(Self::from(PathBuf::from(s))),
        }
    }
}

impl From<PathBuf> for PathOrURI {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

impl From<Url> for PathOrURI {
    fn from(value: Url) -> Self {
        if value.scheme() == "file" {
            Self::from(PathBuf::from(value.path()))
        } else {
            Self::URI(value)
        }
    }
}

impl Display for PathOrURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => write!(f, "{}", path.display()),
            Self::URI(uri) => write!(f, "{}", uri),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path() {
        let path = PathBuf::from("/test/path/dir/");
        assert_eq!(PathOrURI::from(path.clone()), PathOrURI::Path(path));
    }

    #[test]
    fn test_from_web_uri() {
        let url: Url = "https://example.com/subdir/".parse().unwrap();
        assert_eq!(PathOrURI::from(url.clone()), PathOrURI::URI(url),);
    }

    #[test]
    fn test_from_file_uri() {
        let path = PathBuf::from("/test/path");
        let url: Url = "file:///test/path".parse().unwrap();
        assert_eq!(PathOrURI::from(url), PathOrURI::Path(path),);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            PathOrURI::from_str("/test/path").unwrap(),
            PathOrURI::Path(PathBuf::from("/test/path"))
        );

        assert_eq!(
            PathOrURI::from_str("https://example.com").unwrap(),
            PathOrURI::URI("https://example.com".parse().unwrap())
        );

        assert_eq!(
            PathOrURI::from_str("file:///test/path").unwrap(),
            PathOrURI::Path(PathBuf::from("/test/path"))
        );
    }

    #[test]
    fn test_is_uri() {
        assert!(PathOrURI::URI("https://example.com".parse().unwrap()).is_uri());
        assert!(!PathOrURI::Path(PathBuf::from("/test/path")).is_uri());
    }

    #[test]
    fn test_is_path() {
        assert!(!PathOrURI::URI("https://example.com".parse().unwrap()).is_path());
        assert!(PathOrURI::Path(PathBuf::from("/test/path")).is_path());
    }

    #[test]
    fn test_to_uri() {
        let uri: Url = "https://example.com/test/path".parse().unwrap();
        let path = PathBuf::from("./test/next/../file.txt");
        let path_uri =
            Url::from_file_path(std::env::current_dir().unwrap().join(&path).clean()).unwrap();

        assert_eq!(uri.clone(), PathOrURI::URI(uri).uri().unwrap());
        assert_eq!(path_uri, PathOrURI::Path(path).uri().unwrap());
    }
}

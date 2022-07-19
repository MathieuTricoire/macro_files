#![doc = include_str!("../README.md")]

use std::io::{ErrorKind, Result};
use std::path::Path;

#[cfg(feature = "tempfile")]
pub use tempfile;

/// Create persisting directories and files.
///
/// For an example see [library documentation](self)
#[macro_export]
macro_rules! create {
    // Hide distracting implementation details from the generated rustdoc.
    ($($files:tt)+) => {
        {
            #[allow(unused_variables)]
            let path = ::std::path::PathBuf::default();
            $crate::create_internal!(@entries path $($files)+)
        }
    };
}

/// Create directories and files within a temporary directory living the time
/// the returned `tempfile::TempDir` lives.
///
/// # Example
///
/// ```rust
/// let temp_dir = macro_files::create_temp!({
///    "README.md": "# Project name",
///    "LICENSE": "MIT",
/// }).unwrap();
///
/// let file_contents = std::fs::read(temp_dir.path().join("README.md")).unwrap();
/// assert_eq!(
///     String::from_utf8_lossy(&file_contents),
///     "# Project name"
/// );
/// ```
#[cfg(feature = "tempfile")]
#[macro_export]
macro_rules! create_temp {
    // Hide distracting implementation details from the generated rustdoc.
    ($($files:tt)+) => {
        $crate::tempfile::tempdir()
            .and_then(|dir| {
                #[allow(unused_variables)]
                let path = dir.path();
                $crate::create_internal!(@entries path $($files)+).and(Ok(dir))
            })
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! create_internal {
    //
    // Parse entries rules
    //

    // Parse map entries
    (@entries $dir_path:ident { $($files:tt)+ }) => {
        $crate::create_internal!(@entry $dir_path () ($($files)+) ($($files)+))
    };

    // No map entries to parse
    (@entries $dir_path:ident {}) => {
        Ok::<(), ::std::io::Error>(())
    };

    //
    // Parse entry rules
    //

    // Value is null, no file creation.
    (@entry $dir_path:ident ($($file_path:tt)+) (: null $($rest:tt)*) ($($copy:tt)*)) => {
        $crate::create_internal!(@handle $dir_path [$($file_path)+] (false) $($rest)*)
    };

    // Value is false, no file creation.
    (@entry $dir_path:ident ($($file_path:tt)+) (: false $($rest:tt)*) ($($copy:tt)*)) => {
        $crate::create_internal!(@handle $dir_path [$($file_path)+] (false) $($rest)*)
    };

    // Value is true, create an empty file.
    (@entry $dir_path:ident ($($file_path:tt)+) (: true $($rest:tt)*) ($($copy:tt)*)) => {
        $crate::create_internal!(@handle $dir_path [$($file_path)+] (true) $($rest)*)
    };

    // Value is a map with potential entries after.
    // Create map directory, parse the map and then parse the following entries.
    (@entry $dir_path:ident ($($file_path:tt)+) (: { $($map:tt)* } , $($rest:tt)*) ($($copy:tt)*)) => {
        {
            let $dir_path = &$dir_path.join($($file_path)+);
            $crate::create_dir(&$dir_path).and_then(|_| {
                $crate::create_internal!(@entries $dir_path { $($map)* })
            })
        }
        .and_then(|_| $crate::create_internal!(@entries $dir_path { $($rest)* }))
    };

    // Missing comma after a map with following entries.
    (@entry $dir_path:ident ($($file_path:tt)+) (: { $($map:tt)* } $($unexpected:tt)+) (: $($copy:tt)*)) => {
        $crate::create_expect_map_comma!($($copy)*)
    };

    // Value is a map with no entries after.
    // Create map directory and parse the inner map.
    (@entry $dir_path:ident ($($file_path:tt)+) (: { $($map:tt)* }) ($($copy:tt)*)) => {
        {
            let $dir_path = &$dir_path.join($($file_path)+);
            $crate::create_dir(&$dir_path).and_then(|_| {
                $crate::create_internal!(@entries $dir_path { $($map)* })
            })
        }
    };

    // Value is an expression with potential entries after.
    // Handle the entry and parse the following entries.
    (@entry $dir_path:ident ($($file_path:tt)+) (: $contents:expr , $($rest:tt)*) ($($copy:tt)*)) => {
        $crate::create_internal!(@handle $dir_path [$($file_path)+] ($contents) , $($rest)*)
    };

    // Value is an expression with no entries after.
    // Handle the entry.
    (@entry $dir_path:ident ($($file_path:tt)+) (: $contents:expr) ($($copy:tt)*)) => {
        $crate::create_internal!(@handle $dir_path [$($file_path)+] ($contents))
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@entry $dir_path:ident ($($file_path:tt)+) (:) ($($copy:tt)*)) => {
        // "unexpected end of macro invocation"
        $crate::create_internal!()
    };

    // Missing colon and value for last entry. Trigger a reasonable error message.
    (@entry $dir_path:ident ($($file_path:tt)+) () ($($copy:tt)*)) => {
        // "unexpected end of macro invocation"
        $crate::create_internal!()
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@entry $dir_path:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        $crate::create_unexpected!($colon)
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@entry $dir_path:ident ($($file_path:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        $crate::create_unexpected!($comma)
    };

    // Name is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@entry $dir_path:ident () (($file_path:expr) : $($rest:tt)*) ($($copy:tt)*)) => {
        $crate::create_internal!(@entry $dir_path ($file_path) (: $($rest)*) (: $($rest)*))
    };

    // Expect a comma.
    (@entry $dir_path:ident ($($file_path:tt)*) (: $($unexpected:tt)+) ($($copy:tt)*)) => {
        // Expect a comma, so "no rules expected the token `X`".
        $crate::create_expect_comma!($($unexpected)+)
    };

    // Unexpected map before a colon.
    (@entry $dir_path:ident ($($file_path:tt)+) ({ $($map:tt)* } $($rest:tt)*) ($curly_bracket:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `{`".
        $crate::create_unexpected!($curly_bracket)
    };

    // Munch a token into a path.
    (@entry $dir_path:ident ($($path:tt)*) ($tt:tt $($rest:tt)*) ($($copy:tt)*)) => {
        $crate::create_internal!(@entry $dir_path ($($path)* $tt) ($($rest)*) ($($rest)*))
    };

    //
    // Handle rules
    //

    // Handle current entry and continue.
    (@handle $dir_path:ident [$($file_path:tt)+] ($contents:tt) , $($rest:tt)*) => {
        $crate::create_internal!(@write_file ($dir_path) ($($file_path)+) ($contents))
            .and_then(|_| $crate::create_internal!(@entries $dir_path { $($rest)* }))
    };

    // Current entry followed by unexpected token.
    (@handle $dir_path:ident [$($file_path:tt)+] ($contents:expr) $unexpected:tt $($rest:tt)*) => {
        $crate::create_unexpected!($unexpected)
    };

    // Handle current entry and stop.
    (@handle $dir_path:ident [$($file_path:tt)+] ($contents:tt)) => {
        $crate::create_internal!(@write_file ($dir_path) ($($file_path)+) ($contents))
    };

    //
    // Write rules
    //

    // Not write file.
    (@write_file ($dir_path:ident) ($($file_path:tt)+) (false)) => {
        Ok::<(), ::std::io::Error>(())
    };

    // Write an empty file.
    (@write_file ($dir_path:ident) ($($file_path:tt)+) (true)) => {
        $crate::write_file($dir_path.join($($file_path)+), "")
    };

    // Write a file with its contents.
    (@write_file ($dir_path:ident) ($($file_path:tt)+) ($contents:expr)) => {
        $crate::write_file($dir_path.join($($file_path)+), $contents)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! create_expect_comma {
    ($e:expr , $($tt:tt)*) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! create_expect_map_comma {
    ({$($tt:tt)*} , $($rest:tt)*) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! create_unexpected {
    () => {};
}

#[cfg(not(test))]
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    std::fs::create_dir_all(path)
}

#[cfg(test)]
fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    test_helper::create_dir(path)
}

#[cfg(not(test))]
pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    match std::fs::write(&path, &contents) {
        Err(err) if err.kind() == ErrorKind::NotFound => {
            let dir_path = path.as_ref().parent().ok_or(err)?;
            std::fs::create_dir_all(dir_path).and_then(|_| std::fs::write(&path, &contents))
        }
        result => result,
    }
}

#[cfg(test)]
fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    test_helper::write_file(path, contents)
}

#[cfg(test)]
mod test_helper {
    use std::cell::Cell;
    use std::collections::HashSet;
    use std::io::{Error, ErrorKind, Result};
    use std::path::{Path, PathBuf};

    thread_local!(static WRITES: Cell<Option<(Vec<Write>, HashSet<PathBuf>)>> = Cell::new(None));

    pub struct Watcher<T, F>
    where
        T: Sized,
        F: Fn() -> Option<T>,
    {
        consume_cb: F,
    }

    impl<T, F> Watcher<T, F>
    where
        T: Sized,
        F: Fn() -> Option<T>,
    {
        pub fn consume(self) -> T {
            (self.consume_cb)().unwrap()
        }
    }

    impl<T, F> Drop for Watcher<T, F>
    where
        T: Sized,
        F: Fn() -> Option<T>,
    {
        fn drop(&mut self) {
            (self.consume_cb)();
        }
    }

    pub fn watch_fs() -> Watcher<Vec<Write>, impl Fn() -> Option<Vec<Write>>> {
        WRITES.with(|cell| cell.set(Some((Default::default(), Default::default()))));
        Watcher {
            consume_cb: || {
                WRITES
                    .with(|cell| cell.replace(None))
                    .map(|(writes, _)| writes)
            },
        }
    }

    #[derive(PartialEq, Eq, Debug)]
    pub enum Write {
        Dir(PathBuf),
        File(PathBuf, Vec<u8>),
    }

    impl Write {
        pub fn dir(path: impl AsRef<str>) -> Write {
            Write::Dir(path.as_ref().into())
        }

        pub fn file(path: impl AsRef<str>, contents: impl AsRef<str>) -> Write {
            Write::File(path.as_ref().into(), contents.as_ref().into())
        }
    }

    pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        WRITES.with(|cell| {
            if let Some(mut writes) = cell.take() {
                let path = path.as_ref().to_owned();
                if writes.1.contains(&path) {
                    cell.replace(Some(writes));
                    return Err(Error::from(ErrorKind::Other));
                }
                writes.0.push(Write::Dir(path));
                cell.replace(Some(writes));
            }
            Ok(())
        })
    }

    pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
        WRITES.with(|cell| {
            if let Some(mut writes) = cell.take() {
                let path = path.as_ref().to_owned();
                if writes.1.contains(&path) {
                    cell.replace(Some(writes));
                    return Err(Error::from(ErrorKind::Other));
                }
                let contents = contents.as_ref().to_owned();
                writes.0.push(Write::File(path, contents));
                cell.replace(Some(writes));
            }
            Ok(())
        })
    }

    pub fn fail_fs<P: AsRef<Path>>(path: P) {
        WRITES.with(|cell| {
            if let Some(mut writes) = cell.take() {
                writes.1.insert(path.as_ref().to_owned());
                cell.replace(Some(writes));
            }
        });
    }
}

#[cfg(test)]
mod fs_tests {
    use super::test_helper::Write;
    use super::*;

    #[test]
    fn test_1() {
        let watcher = test_helper::watch_fs();
        create!({
            "README.md": "# Project",
            ("LICENSE"): "MIT"
        })
        .unwrap();
        let expected = vec![
            Write::file("README.md", "# Project"),
            Write::file("LICENSE", "MIT"),
        ];
        assert_eq!(watcher.consume(), expected);
    }

    #[test]
    fn test_2() {
        let watcher = test_helper::watch_fs();
        create!({
            "directory": {
                "README.md": "# Project"
            },
            "sibling": {}
        })
        .unwrap();
        let expected = vec![
            Write::dir("directory"),
            Write::file("directory/README.md", "# Project"),
            Write::dir("sibling"),
        ];
        assert_eq!(watcher.consume(), expected);
    }

    #[test]
    fn test_3() {
        let watcher = test_helper::watch_fs();

        let project_name = String::from("Rust project");
        let adr_directory = "adr";
        let adr_template = ["# NUMBER. TITLE", "", "Date: DATE"].join("\n");
        fn license() -> &'static str {
            "MIT License..."
        }
        fn markdown(name: &str) -> String {
            format!("{}.md", name)
        }

        create!({
            ["long", "path"].join("/"): {
                markdown("README"): format!("# {}", project_name),
                "docs": {
                    markdown("README"): "# Documentation",
                    "assets": {},
                    "examples": {}
                },
                adr_directory: {
                    "templates": {
                        markdown("template"): adr_template,
                    }
                },
                "LICENSE": license(),
                ".adr-dir": adr_directory,
            },
            "other": {
                "not-create-1": false,
                "not-create-2": null,
                ".gitkeep": true,
                "path/as/name": true
            },
        })
        .unwrap();
        let expected = vec![
            Write::dir("long/path"),
            Write::file("long/path/README.md", "# Rust project"),
            Write::dir("long/path/docs"),
            Write::file("long/path/docs/README.md", "# Documentation"),
            Write::dir("long/path/docs/assets"),
            Write::dir("long/path/docs/examples"),
            Write::dir("long/path/adr"),
            Write::dir("long/path/adr/templates"),
            Write::file(
                "long/path/adr/templates/template.md",
                "# NUMBER. TITLE\n\nDate: DATE",
            ),
            Write::file("long/path/LICENSE", "MIT License..."),
            Write::file("long/path/.adr-dir", "adr"),
            Write::dir("other"),
            Write::file("other/.gitkeep", ""),
            Write::file("other/path/as/name", ""),
        ];
        assert_eq!(watcher.consume(), expected);
    }

    #[test]
    fn directory_fails() {
        let watcher = test_helper::watch_fs();
        test_helper::fail_fs("second-error");
        let result = create!({
            "first-success": {},
            "second-error": {},
            "third-not-attempted": {},
        });
        let expected = vec![Write::dir("first-success")];
        assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
        assert_eq!(watcher.consume(), expected);

        let watcher = test_helper::watch_fs();
        test_helper::fail_fs("second-error");
        let result = create!({
            "first-success": {
                "README.md": "# Project 1",
            },
            "second-error": {
                "README.md": "# Project 2",
            }
        });
        let expected = vec![
            Write::dir("first-success"),
            Write::file("first-success/README.md", "# Project 1"),
        ];
        assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
        assert_eq!(watcher.consume(), expected);
    }

    #[test]
    fn file_fails() {
        let watcher = test_helper::watch_fs();
        test_helper::fail_fs("second-success/README.md");
        let result = create!({
            "first-success": {
                "README.md": "# Project 1",
                "LICENSE": "MIT"
            },
            "second-success": {
                "README.md": "# Project error",
                "LICENSE": "MIT"
            },
            "third-not-attempted": {
                "README.md": "# Project 3",
                "LICENSE": "MIT"
            },
        });
        let expected = vec![
            Write::dir("first-success"),
            Write::file("first-success/README.md", "# Project 1"),
            Write::file("first-success/LICENSE", "MIT"),
            Write::dir("second-success"),
        ];
        assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
        assert_eq!(watcher.consume(), expected);
    }
}

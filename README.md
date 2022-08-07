# macro_files

[![CI badge]][CI] [![Crate badge]][Crate] [![Rustc badge]][Rustc]

[CI badge]: https://img.shields.io/github/workflow/status/MathieuTricoire/macro_files/CI/main
[CI]: https://github.com/MathieuTricoire/macro_files/actions?query=branch%3Amain
[Crate badge]: https://img.shields.io/crates/v/macro_files.svg
[Crate]: https://crates.io/crates/macro_files
[Rustc badge]: https://img.shields.io/badge/rustc-1.56+-lightgray.svg
[Rustc]: https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html

_Macro consuming JSON like data structures to create directories and files at runtime._

## Installation

```toml
[dependencies]
macro_files = "0.1"
```

Version requirement: rustc 1.56+

## Examples

Create directories and files

```rust
fn project_readme(project_name: &str) -> String {
    format!("# {}", project_name)
}
let project_name = "Project name".to_string();
let adr_directory = "adr";

let temp_dir = macro_files::tempfile::tempdir().unwrap();
macro_files::create!({
   temp_dir.path(): {
        "README.md": project_readme(&project_name),
        ".adr-dir": adr_directory,
        adr_directory: {
            "templates": {
                "template.md": "# ADR Template",
            }
        },
        "LICENSE": "MIT"
    },
}).unwrap();

let file_contents = std::fs::read(temp_dir.path().join("LICENSE")).unwrap();
assert_eq!(
    String::from_utf8_lossy(&file_contents),
    "MIT"
);

// Macro expands as:
// {
//     #[allow(unused_variables)]
//     let path = ::std::path::PathBuf::default();
//     {
//         let path = &path.join(temp_dir.path());
//         $crate::create_dir(&path).and_then(|_| {
//             $crate::write_file(path.join("README.md"), (project_readme(&project_name)))
//                 .and_then(|_| {
//                     $crate::write_file(path.join(".adr-dir"), adr_directory).and_then(|_| {
//                         {
//                             let path = &path.join(adr_directory);
//                             $crate::create_dir(&path).and_then(|_| {
//                                 let path = &path.join("templates");
//                                 $crate::create_dir(&path).and_then(|_| {
//                                     $crate::write_file(
//                                         path.join("template.md"),
//                                         "# ADR Template",
//                                     )
//                                     .and_then(|_| Ok::<(), ::std::io::Error>(()))
//                                 })
//                             })
//                         }
//                         .and_then(|_| $crate::write_file(path.join("LICENSE"), "MIT"))
//                     })
//                 })
//         })
//     }
//     .and_then(|_| Ok::<(), ::std::io::Error>(()))
// }
```

Create directories and files within a temporary directory.

_This requires the default feature `tempfile` that uses the [`tempfile`] crate._

The macro will return a [`tempfile::TempDir`] struct, the temporary directory will lives as long as
the returned [`tempfile::TempDir`] struct is not dropped (see documentation).

```rust
let temp_dir = macro_files::create_temp!({
   "README.md": "# Project name",
   "LICENSE": "MIT",
}).unwrap();

let file_contents = std::fs::read(temp_dir.path().join("README.md")).unwrap();
assert_eq!(
    String::from_utf8_lossy(&file_contents),
    "# Project name"
);
```

---

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`tempfile`]: https://crates.io/crates/tempfile
[`tempfile::TempDir`]: https://docs.rs/tempfile/3.3.0/tempfile/struct.TempDir.html

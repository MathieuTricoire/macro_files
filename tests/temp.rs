use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[test]
fn temp() {
    let project_name = String::from("Rust project");
    let adr_directory = "adr";
    let adr_template = ["# NUMBER. TITLE", "", "Date: DATE"].join("\n");
    fn license() -> &'static str {
        "MIT"
    }
    fn markdown(name: &str) -> String {
        format!("{}.md", name)
    }

    let dir = macro_files::create_temp!({
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
            "path/as/file-name": "file path",
            "path": {
                "file": "existing path"
            }
        },
    })
    .unwrap();

    let expected = HashSet::from([
        Entry::dir("long"),
        Entry::dir("long/path"),
        Entry::file("long/path/README.md", "# Rust project"),
        Entry::dir("long/path/docs"),
        Entry::file("long/path/docs/README.md", "# Documentation"),
        Entry::dir("long/path/docs/assets"),
        Entry::dir("long/path/docs/examples"),
        Entry::dir("long/path/adr"),
        Entry::dir("long/path/adr/templates"),
        Entry::file(
            "long/path/adr/templates/template.md",
            "# NUMBER. TITLE\n\nDate: DATE",
        ),
        Entry::file("long/path/LICENSE", "MIT"),
        Entry::file("long/path/.adr-dir", "adr"),
        Entry::dir("other"),
        Entry::file("other/.gitkeep", ""),
        Entry::dir("other/path"),
        Entry::dir("other/path/as"),
        Entry::file("other/path/as/file-name", "file path"),
        Entry::file("other/path/file", "existing path"),
    ]);
    assert_eq!(get_entries(dir.path(), &PathBuf::default()), expected);
}

#[derive(Hash, PartialEq, Eq, Debug)]
enum Entry {
    Dir(PathBuf),
    File(PathBuf, Vec<u8>),
}

impl Entry {
    fn dir(path: &str) -> Entry {
        Entry::Dir(path.into())
    }

    fn file(path: &str, contents: &str) -> Entry {
        Entry::File(path.into(), contents.into())
    }
}

fn get_entries(path: &Path, relative_path: &Path) -> HashSet<Entry> {
    let mut entries = HashSet::new();
    for entry in path.read_dir().unwrap() {
        let entry_path = entry.unwrap().path();
        let relative_path = relative_path.join(entry_path.strip_prefix(path).unwrap());
        if entry_path.is_dir() {
            entries.insert(Entry::Dir(relative_path.clone()));
            for entry in get_entries(&entry_path, &relative_path) {
                entries.insert(entry);
            }
        } else if entry_path.is_file() {
            entries.insert(Entry::File(
                relative_path,
                std::fs::read(entry_path).unwrap(),
            ));
        }
    }
    entries
}

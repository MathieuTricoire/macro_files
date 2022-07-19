use macro_files::create;

fn main() {
    create!({
        "README.md": "# Title"
        ".gitignore": "/target"
    });
}

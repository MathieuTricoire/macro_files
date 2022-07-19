use macro_files::create;

fn main() {
    create!({
        "README.md",
        ".gitignore": "/target"
    });
}

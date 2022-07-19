use macro_files::create;

fn main() {
    create!({
        "docs": {
            "README.md": "# Title"
        }
        "LICENSE": "MIT"
    });
}

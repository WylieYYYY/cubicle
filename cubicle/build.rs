use std::fs::{self, File};
use std::path::Path;

use fs_extra::dir::CopyOptions;
use tera::{Context, Tera};

fn main() {
    let build_dir = Path::new("../build/");
    let resource_dir = Path::new("res/");
    let context_file = Path::new("components/context.js");

    if !build_dir.exists() { fs::create_dir(build_dir).unwrap(); }
    for entry in fs::read_dir(resource_dir).unwrap() {
        fs_extra::copy_items(&vec![entry.unwrap().path()],
            build_dir, &CopyOptions::default().overwrite(true)).unwrap();
    }

    let mut tera = Tera::default();
    let mut context = Context::new();
    let mut view_names = Vec::new();
    let context_filename = context_file.file_stem().unwrap().to_string_lossy();
    let context_extension = context_file.extension().unwrap().to_string_lossy();

    for entry in fs::read_dir(build_dir.join(context_file)
        .parent().unwrap()).unwrap() {
        let entry = entry.unwrap();

        let entry_filename = entry.file_name();
        let entry_filename = entry_filename.to_string_lossy();

        if let Some(view_name) = entry_filename.strip_suffix(
            &(String::from(".") + &context_extension)) {
            if view_name == context_filename { continue; }
            view_names.push(String::from(view_name));
        }
    }

    tera.add_template_file(resource_dir.join(context_file),
        Some(&context_filename)).unwrap();
    context.insert("view_names", &view_names);
    let context_file = &File::create(build_dir
        .join(context_file)).unwrap();
    tera.render_to(&context_filename, &context, context_file).unwrap();
}

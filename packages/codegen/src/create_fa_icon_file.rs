use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use codegen::Scope;
use heck::ToUpperCamelCase;
use scraper::Html;
use scraper::Selector;
use walkdir::WalkDir;

#[derive(Debug)]
struct Icon {
    name: String,
    view_box: String,
    xmlns: String,
    d: String,
}

fn icon_name(path: &PathBuf) -> String {
    let filename = path.file_name().unwrap().to_str().unwrap();
    let name = filename.split('.').next().unwrap();
    name.to_upper_camel_case()
}

pub fn create_fa_icon_file(svg_path: &str, output_path: &str) {
    let dir_entries = WalkDir::new(svg_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();

    let files = dir_entries
        .into_iter()
        .filter(|e| e.path().extension() == Some(OsStr::new("svg")))
        .map(|dir| PathBuf::from(dir.path()))
        .collect::<Vec<_>>();

    let mut icons = Vec::new();
    let svg_selector = Selector::parse("svg").unwrap();
    let path_selector = Selector::parse("path").unwrap();

    for file in files {
        let svg_str = fs::read_to_string(&file).unwrap();
        let fragment = Html::parse_fragment(&svg_str);
        let svg_data = fragment.select(&svg_selector).next().unwrap();
        let path_data = fragment.select(&path_selector).next().unwrap();

        icons.push(Icon {
            name: icon_name(&file),
            view_box: svg_data.value().attr("viewBox").unwrap().to_string(),
            xmlns: "http://www.w3.org/2000/svg".to_string(),
            d: path_data.value().attr("d").unwrap().to_string(),
        })
    }

    let mut scope = Scope::new();
    scope.raw("use super::super::Icon;");

    // add icon data
    for icon in icons.iter() {
        scope.raw(&format!(
            "#[allow(dead_code, non_upper_case_globals)]
pub const Fa{}: Icon = Icon {{
    view_box: \"{}\",
    xmlns: \"{}\",
    d: \"{}\",
}};",
            icon.name, icon.view_box, icon.xmlns, icon.d
        ));
    }

    // write to file
    let mut file = File::create(output_path).unwrap();
    file.write_all(scope.to_string().as_bytes()).unwrap();
    file.flush().unwrap();
}
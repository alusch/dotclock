use std::fs;

use eg_font_converter::FontConverter;

const FONTS_DIR: &str = "src/fonts";

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    for entry in fs::read_dir(FONTS_DIR).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "bdf") {
            let font_name =
                "FONT_".to_owned() + &path.file_stem().unwrap().to_string_lossy().to_uppercase();
            let font = FontConverter::new(path, &font_name)
                .convert_eg_bdf()
                .unwrap();
            font.save(&out_dir).unwrap();
        }
    }

    println!("cargo:rerun-if-changed={}", FONTS_DIR);
    println!("cargo:rerun-if-changed=build.rs");
}

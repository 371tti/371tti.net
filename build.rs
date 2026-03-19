use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url =
        "http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict/sudachi-dictionary-20260116-full.zip";

    let _out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let static_dir = PathBuf::from("static");
    let target_path = static_dir.join("system.dic");

    println!("cargo:rerun-if-changed=build.rs");

    fs::create_dir_all(&static_dir)?;

    let bytes = reqwest::blocking::get(url)?.bytes()?;
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)?;

    let wanted = Path::new("sudachi-dictionary-20260116/system_full.dic");

    let mut entry = archive.by_name(
        wanted
            .to_str()
            .ok_or("invalid target path in zip entry name")?,
    )?;

    let mut out = fs::File::create(&target_path)?;
    std::io::copy(&mut entry, &mut out)?;

    println!("cargo:warning=Copied Sudachi dictionary to {}", target_path.display());

    Ok(())
}
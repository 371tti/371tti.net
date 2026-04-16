use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

const DICT_URLS: [&str; 2] = [
    "https://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict/sudachi-dictionary-20260116-full.zip",
    "http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict/sudachi-dictionary-20260116-full.zip",
];
const WANTED_PATH: &str = "sudachi-dictionary-20260116/system_full.dic";
const WANTED_SUFFIX: &str = "/system_full.dic";
const RETRY_COUNT: usize = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let static_dir = PathBuf::from("static");
    let target_path = static_dir.join("system.dic");

    println!("cargo:rerun-if-changed=build.rs");

    fs::create_dir_all(&static_dir)?;

    if target_path.exists() && fs::metadata(&target_path)?.len() > 0 {
        println!(
            "cargo:warning=Using existing Sudachi dictionary at {}",
            target_path.display()
        );
        return Ok(());
    }

    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(300))
        .build()?;

    let mut last_error: Option<String> = None;

    for url in DICT_URLS {
        for attempt in 1..=RETRY_COUNT {
            match download_dict(&client, url, &target_path) {
                Ok(()) => {
                    println!(
                        "cargo:warning=Copied Sudachi dictionary to {}",
                        target_path.display()
                    );
                    return Ok(());
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                    println!(
                        "cargo:warning=Failed to download Sudachi dictionary from {} (attempt {}/{}): {}",
                        url, attempt, RETRY_COUNT, err
                    );
                    if attempt < RETRY_COUNT {
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
        }
    }

    let mut message = String::from("Failed to prepare static/system.dic.");
    if let Some(err) = last_error {
        message.push_str(&format!(" Last error: {err}."));
    }
    message
        .push_str(" Place a Sudachi dictionary manually at static/system.dic and run cargo again.");

    Err(message.into())
}

fn download_dict(
    client: &reqwest::blocking::Client,
    url: &str,
    target_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(url).send()?.error_for_status()?;
    let bytes = response.bytes()?.to_vec();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;

    let entry_index = match archive.index_for_name(WANTED_PATH) {
        Some(index) => index,
        None => find_entry_index_by_suffix(&mut archive, WANTED_SUFFIX)
            .ok_or("Could not find system_full.dic in downloaded archive")?,
    };

    let mut entry = archive.by_index(entry_index)?;
    let temp_path = target_path.with_extension("dic.part");

    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }

    {
        let mut out = fs::File::create(&temp_path)?;
        std::io::copy(&mut entry, &mut out)?;
    }

    if target_path.exists() {
        fs::remove_file(target_path)?;
    }
    fs::rename(&temp_path, target_path)?;

    Ok(())
}

fn find_entry_index_by_suffix<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    suffix: &str,
) -> Option<usize> {
    for i in 0..archive.len() {
        let name = match archive.by_index(i) {
            Ok(file) => file.name().to_owned(),
            Err(_) => continue,
        };

        if name.ends_with(suffix) {
            return Some(i);
        }
    }

    None
}

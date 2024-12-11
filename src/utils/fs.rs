use std::path::PathBuf;

pub fn get_file_path(relative_path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let binding = std::env::current_exe()?;
    let exe_dir = binding.parent().ok_or("Failed to get executable directory")?;
    Ok(exe_dir.join(relative_path))
}
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn test_dir() -> PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("itty_bitty_tests_{}", id));
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn readme_content() -> &'static str {
    "# itty-bitty test file\n\nThis is test content for archive format testing.\n"
}

pub fn create_readme(dir: &PathBuf) -> PathBuf {
    let path = dir.join("README.md");
    fs::write(&path, readme_content()).expect("Failed to write README.md");
    path
}

pub fn has_command(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn create_gzip() -> Option<PathBuf> {
    if !has_command("gzip") {
        println!("⏭ gzip not found, skipping");
        return None;
    }
    let dir = test_dir();
    let readme = create_readme(&dir);
    let output = dir.join("README.md.gz");
    let status = Command::new("gzip")
        .args(["-k", "-f"])
        .arg(&readme)
        .status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("✓ gzip found");
        Some(output)
    } else {
        println!("✗ gzip failed");
        None
    }
}

pub fn create_bzip2() -> Option<PathBuf> {
    if !has_command("bzip2") {
        println!("⏭ bzip2 not found, skipping");
        return None;
    }
    let dir = test_dir();
    let readme = create_readme(&dir);
    let output = dir.join("README.md.bz2");
    let status = Command::new("bzip2")
        .args(["-k", "-f"])
        .arg(&readme)
        .status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("✓ bzip2 found");
        Some(output)
    } else {
        println!("✗ bzip2 failed");
        None
    }
}

pub fn create_zip() -> Option<PathBuf> {
    if !has_command("zip") {
        println!("⏭ zip not found, skipping");
        return None;
    }
    let dir = test_dir();
    let readme = create_readme(&dir);
    let output = dir.join("README.zip");
    let _ = fs::remove_file(&output); // zip won't overwrite
    let status = Command::new("zip")
        .arg(&output)
        .arg(&readme)
        .current_dir(&dir)
        .status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("✓ zip found");
        Some(output)
    } else {
        println!("✗ zip failed");
        None
    }
}

pub fn create_tar() -> Option<PathBuf> {
    if !has_command("tar") {
        println!("⏭ tar not found, skipping");
        return None;
    }
    let dir = test_dir();
    let _readme = create_readme(&dir);
    let output = dir.join("README.tar");
    let status = Command::new("tar")
        .args(["-cf"])
        .arg(&output)
        .arg("-C")
        .arg(&dir)
        .arg("README.md")
        .status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("✓ tar found");
        Some(output)
    } else {
        println!("✗ tar failed");
        None
    }
}

pub fn create_tar_xz() -> Option<PathBuf> {
    if !has_command("tar") {
        println!("⏭ tar not found, skipping");
        return None;
    }
    let dir = test_dir();
    let _readme = create_readme(&dir);
    let output = dir.join("README.tar.xz");
    let status = Command::new("tar")
        .args(["-cJf"])
        .arg(&output)
        .arg("-C")
        .arg(&dir)
        .arg("README.md")
        .status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("✓ tar (xz) found");
        Some(output)
    } else {
        println!("⏭ tar xz compression not available, skipping");
        None
    }
}

pub fn read_bits(file: &str, offset: i64, bits: usize, format: &str) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_itty-bitty"));
    cmd.arg("-f").arg(format).arg(file);
    // Add -- before negative offsets to prevent clap from treating them as flags
    if offset < 0 {
        cmd.arg("--");
    }
    cmd.arg(offset.to_string()).arg(bits.to_string());
    let output = cmd.output().expect("Failed to run itty-bitty");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn read_bits_hex(file: &str, offset: i64, bits: usize) -> String {
    read_bits(file, offset, bits, "hex")
}

pub fn read_bits_decimal(file: &str, offset: i64, bits: usize) -> u64 {
    read_bits(file, offset, bits, "decimal").parse().unwrap_or(0)
}

pub fn read_bits_decimal_lsb(file: &str, offset: i64, bits: usize) -> u64 {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_itty-bitty"));
    cmd.arg("-e").arg("lsb").arg("-f").arg("decimal").arg(file);
    // Add -- before negative offsets to prevent clap from treating them as flags
    if offset < 0 {
        cmd.arg("--");
    }
    cmd.arg(offset.to_string()).arg(bits.to_string());
    let output = cmd.output().expect("Failed to run itty-bitty");
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0)
}

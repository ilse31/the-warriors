use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub const ELF_SHA256: &str = "1a4e1c3891c930146e39ce340ccb02918cfe917032aafcdbbbe6541f75402369";
const MEMBERS: &[&str] = &[
    "SYSTEM.CNF",
    "SLUS_212.15",
    "WARRIORS.DIR",
    "WARRIORS.WAD",
    "MODULES/IOP.IRX",
    "MODULES/IOPRP300.IMG",
    "MODULES/LIBSD.IRX",
    "MODULES/MCMAN.IRX",
    "MODULES/MCSERV.IRX",
    "MODULES/PADMAN.IRX",
    "MODULES/SIO2MAN.IRX",
    "IOP/BFW.SND",
    "IOP/MUSIC.SND",
    "PSS/L1_IN.BIK",
    "PSS/L31_IN.BIK",
    "PSS/L31_OUT.BIK",
    "PSS/L34_OUT.BIK",
    "PSS/L51_IN.BIK",
    "PSS/L52_IN.BIK",
    "PSS/L54_IN.BIK",
    "PSS/L81_OUT.BIK",
    "PSS/L84_OUT.BIK",
    "PSS/L87_OUT.BIK",
    "PSS/L99_IN.BIK",
    "PSS/L9_IN.BIK",
    "PSS/L9_OUT.BIK",
    "PSS/LOGO.BIK",
    "PSS/PLOGO.BIK",
    "PSS/TRAILER.BIK",
];

pub fn sha256_file(path: &Path) -> Result<String, String> {
    if !fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_file()
    {
        return Err("File game harus berupa file biasa, bukan symlink atau folder.".into());
    }
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn verify_elf(path: &Path) -> Result<(), String> {
    if sha256_file(path)? != ELF_SHA256 {
        return Err("ISO tidak cocok. Paket ini hanya mendukung The Warriors USA SLUS_212.15 dengan executable yang terverifikasi.".into());
    }
    Ok(())
}

fn extract(iso: &Path, directory: &Path, members: &[&str]) -> Result<(), String> {
    #[cfg(windows)]
    let mut command = Command::new(
        PathBuf::from(std::env::var_os("SystemRoot").ok_or("SystemRoot Windows tidak tersedia.")?)
            .join("System32/tar.exe"),
    );
    #[cfg(not(windows))]
    let mut command = Command::new("tar");
    command
        .arg("-xf")
        .arg(iso)
        .arg("-C")
        .arg(directory)
        .arg("--")
        .args(members);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command
        .output()
        .map_err(|e| format!("Tidak dapat menjalankan tar bawaan Windows: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "Ekstraksi gagal. Periksa ISO dan ruang disk. {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub fn verify_installation(root: &Path) -> Result<(), String> {
    let data = root.join("data");
    verify_elf(&data.join("SLUS_212.15"))?;
    for member in MEMBERS {
        let metadata = fs::symlink_metadata(data.join(member))
            .map_err(|e| format!("Aset {member} tidak tersedia: {e}"))?;
        if !metadata.file_type().is_file() || metadata.len() == 0 {
            return Err(format!("Aset {member} tidak valid."));
        }
    }
    Ok(())
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        #[cfg(windows)]
        let _ = clear_readonly(&self.0);
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(windows)]
#[allow(
    clippy::permissions_set_readonly_false,
    reason = "Windows-only: clears the DOS read-only attribute on owned staging files, without changing Unix mode bits"
)]
fn clear_readonly(path: &Path) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            clear_readonly(&entry?.path())?;
        }
    } else if metadata.permissions().readonly() {
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

pub fn import_iso(iso: &Path, installations: &Path) -> Result<PathBuf, String> {
    let iso = iso
        .canonicalize()
        .map_err(|e| format!("ISO tidak ditemukan: {e}"))?;
    if !iso.is_file()
        || !iso
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("iso"))
    {
        return Err("Pilih file ISO PS2 (.iso).".into());
    }
    fs::create_dir_all(installations).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let stage = Staging(installations.join(format!(".import-{}-{stamp}", std::process::id())));
    fs::create_dir(&stage.0).map_err(|e| e.to_string())?;
    let data = stage.0.join("data");
    fs::create_dir(&data).map_err(|e| e.to_string())?;
    extract(&iso, &data, &["SLUS_212.15"])?;
    verify_elf(&data.join("SLUS_212.15"))?;
    let installed = installations.join(ELF_SHA256);
    if installed.exists() {
        verify_installation(&installed)?;
        return Ok(installed);
    }
    let remaining: Vec<_> = MEMBERS
        .iter()
        .copied()
        .filter(|name| *name != "SLUS_212.15")
        .collect();
    extract(&iso, &data, &remaining)?;
    verify_installation(&stage.0)?;
    fs::rename(&stage.0, &installed).map_err(|e| format!("Instalasi tidak dapat disimpan: {e}"))?;
    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes_known_bytes_and_rejects_a_wrong_game_build() {
        let f = tempfile::NamedTempFile::new().unwrap();
        assert_eq!(
            sha256_file(f.path()).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert!(verify_elf(f.path()).unwrap_err().contains("SLUS_212.15"));
    }

    #[test]
    fn bad_iso_preserves_existing_installation() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("installed");
        fs::create_dir(&old).unwrap();
        fs::write(old.join("save.dat"), b"keep my save").unwrap();
        let iso = root.path().join("broken.iso");
        fs::write(&iso, b"not an ISO").unwrap();
        assert!(import_iso(&iso, &old).is_err());
        assert_eq!(fs::read(old.join("save.dat")).unwrap(), b"keep my save");
        assert_eq!(fs::read_dir(&old).unwrap().count(), 1);
    }

    #[test]
    fn rejects_directory_and_non_iso_inputs() {
        let root = tempfile::tempdir().unwrap();
        let other = root.path().join("game.exe");
        fs::write(&other, b"file").unwrap();
        assert!(import_iso(root.path(), &root.path().join("install")).is_err());
        assert!(import_iso(&other, &root.path().join("install")).is_err());
        assert!(!root.path().join("install").exists());
    }

    #[cfg(windows)]
    #[test]
    fn removes_readonly_staging_files_without_touching_other_files() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("save.dat"), b"keep").unwrap();
        let stage = Staging(root.path().join("staging"));
        fs::create_dir(&stage.0).unwrap();
        let file = stage.0.join("SLUS_212.15");
        fs::write(&file, b"test").unwrap();
        let mut permissions = fs::metadata(&file).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(file, permissions).unwrap();
        drop(stage);
        assert!(!root.path().join("staging").exists());
        assert_eq!(fs::read(root.path().join("save.dat")).unwrap(), b"keep");
    }

    #[test]
    #[ignore = "requires a user-provided ISO and approximately 5 GB of temporary disk space"]
    fn supported_iso_imports_and_preserves_saves_on_reimport() {
        let iso =
            PathBuf::from(std::env::var_os("WARRIORS_TEST_ISO").expect("Set WARRIORS_TEST_ISO"));
        let root = tempfile::tempdir().unwrap();
        let installed = import_iso(&iso, root.path()).unwrap();
        fs::write(installed.join("save-test.dat"), b"keep").unwrap();
        assert_eq!(import_iso(&iso, root.path()).unwrap(), installed);
        assert_eq!(fs::read(installed.join("save-test.dat")).unwrap(), b"keep");
        verify_installation(&installed).unwrap();
    }
}

use std::{path::Path, process::Command};
pub mod install;

pub fn prepare_command(game: &Path, launcher: &Path) -> Result<Command, String> {
    let game = game
        .canonicalize()
        .map_err(|e| format!("File game tidak ditemukan: {e}"))?;
    let launcher = launcher
        .canonicalize()
        .map_err(|e| format!("Lokasi launcher tidak dapat diperiksa: {e}"))?;
    if game == launcher {
        return Err("Pilih executable game, bukan launcher sendiri.".into());
    }
    if !game.is_file()
        || !game
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Err(
            "Pilih file game Windows (.exe). ISO PS2 tidak dapat dijalankan langsung.".into(),
        );
    }
    let directory = game.parent().ok_or("Folder game tidak dapat ditentukan.")?;
    let mut command = Command::new(&game);
    command.current_dir(directory);
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("warriors-{}-{name}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn file(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, b"test fixture").unwrap();
            path
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn launches_exact_path_with_spaces_from_game_directory() {
        let f = Fixture::new("spaces");
        let game = f.file("The Warriors.EXE");
        let launcher = std::env::current_exe().unwrap();
        let command = prepare_command(&game, &launcher).unwrap();
        assert_eq!(command.get_program(), fs::canonicalize(&game).unwrap());
        assert_eq!(
            command.get_current_dir(),
            Some(f.0.canonicalize().unwrap().as_path())
        );
        assert_eq!(command.get_args().count(), 0);
    }

    #[test]
    fn rejects_iso_instead_of_pretending_it_is_a_native_game() {
        let f = Fixture::new("iso");
        let iso = f.file("The Warriors.iso");
        let error = prepare_command(&iso, &std::env::current_exe().unwrap()).unwrap_err();
        assert!(error.contains(".exe"));
    }

    #[test]
    fn rejects_launcher_itself_to_prevent_recursive_spawning() {
        let launcher = std::env::current_exe().unwrap();
        let error = prepare_command(&launcher, &launcher).unwrap_err();
        assert!(error.contains("launcher sendiri"));
    }

    #[test]
    fn rejects_missing_file() {
        let f = Fixture::new("missing");
        assert!(
            prepare_command(&f.0.join("missing.exe"), &std::env::current_exe().unwrap()).is_err()
        );
    }

    #[test]
    fn rejects_directory_with_exe_extension() {
        let f = Fixture::new("directory");
        let dir = f.0.join("game.exe");
        fs::create_dir(&dir).unwrap();
        assert!(prepare_command(&dir, &std::env::current_exe().unwrap()).is_err());
    }

    #[test]
    fn starts_a_real_process_and_observes_successful_exit() {
        let f = Fixture::new("process");
        let launcher = std::env::current_exe().unwrap();
        let game = f.0.join("Test Game.exe");
        fs::copy(&launcher, &game).unwrap();
        let output = prepare_command(&game, &launcher)
            .unwrap()
            .arg("--list")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.is_empty());
    }
}

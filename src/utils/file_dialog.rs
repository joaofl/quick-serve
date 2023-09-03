use std::path::{PathBuf};
use rfd;


pub fn show_open_dialog(dir: PathBuf) -> PathBuf {
    let mut dialog = rfd::FileDialog::new();
    dialog = dialog.set_title("Select a directory to serve");

    // if let Some(directory) = dir {
    //     dialog = dialog.set_directory(directory);
    // }

    match dialog.pick_folder() {
        Some(new_path) => new_path.into(),
        None => dir,
    }
}


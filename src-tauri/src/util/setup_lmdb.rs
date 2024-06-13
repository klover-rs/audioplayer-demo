use std::fs;
use tracing::error;

pub fn lmdb_data_folder() -> String {
    let mut lmdb_data_folder = String::new();

    if let Some(home_dir) = dirs::home_dir() {
        let folder_path = home_dir.join("lmdb_data");

        if !folder_path.exists() {
            if let Err(err) = fs::create_dir(&folder_path) {
                error!("failed the create folder: {}", err)
            }
        }
        let path_string = folder_path
            .into_os_string()
            .into_string()
            .unwrap_or_else(|os_string| os_string.to_string_lossy().into_owned());

        lmdb_data_folder.push_str(&path_string);

    } else {
        error!("unable to get the document directory");
    }

    

    lmdb_data_folder
}
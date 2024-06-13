extern crate lmdb_rs as lmdb;

use lmdb::{EnvBuilder, DbFlags, core::MdbError};
use std::path::PathBuf;

use anyhow::Result;

use crate::util::setup_lmdb::lmdb_data_folder;

pub fn store_songs_directory(dir: &str) -> Result<()> {
    let data_folder = lmdb_data_folder();

    let data_folder_pathbuf = PathBuf::from(data_folder);

    let env = EnvBuilder::new()
        .open(data_folder_pathbuf, 0o777)?;

    let db_handle = env.get_default_db(DbFlags::empty())?;

    let txn = env.new_transaction()?;
    {
        let db = txn.bind(&db_handle);
        db.set(&"song-dir", &dir)?;
    }

    txn.commit()?;

    Ok(())
}

pub fn get_songs_directory() -> Result<Option<String>> {
    let data_folder = lmdb_data_folder();

    let data_folder_pathbuf = PathBuf::from(data_folder);

    let env = EnvBuilder::new()
        .open(data_folder_pathbuf, 0o777)?;

    let db_handle = env.get_default_db(DbFlags::empty())?;

    let reader =env.get_reader()?;

    let db = reader.bind(&db_handle);

    match db.get::<String>(&"song-dir") {
        Ok(song_dir) => Ok(Some(song_dir)),
        Err(e) => {
            match e {
                MdbError::NotFound => Ok(None),
                _ => Err(anyhow::Error::new(e).context("failed to get song dir from lmdb"))
            }
        }
    }

}
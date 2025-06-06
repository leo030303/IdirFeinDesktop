use keepass::{
    db::{Entry, NodeRef, Value},
    Database, DatabaseKey,
};
use std::{fs::File, path::PathBuf};

use super::page::Password;

pub async fn get_passwords(
    keepass_file_path: PathBuf,
    master_password_attempt: Option<String>,
    keyfile_option: Option<PathBuf>,
) -> Result<Vec<Password>, String> {
    let mut file = File::open(keepass_file_path).map_err(|err| err.to_string())?;
    let mut key = DatabaseKey::new();
    if let Some(master_password) = master_password_attempt {
        key = key.with_password(&master_password);
    };
    if let Some(keyfile) = keyfile_option {
        key = key
            .with_keyfile(&mut File::open(keyfile).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    }

    Ok(Database::open(&mut file, key)
        .map_err(|err| err.to_string())?
        .root
        .iter()
        .filter_map(|node| {
            if let NodeRef::Entry(entry) = node {
                Some(Password {
                    id: entry.uuid,
                    title: String::from(entry.get_title().unwrap_or("")),
                    username: String::from(entry.get_username().unwrap_or("")),
                    url: String::from(entry.get_url().unwrap_or("")),
                    password: String::from(entry.get_password().unwrap_or("")),
                })
            } else {
                None
            }
        })
        .collect::<Vec<Password>>())
}

pub async fn save_database(
    database_path: Option<PathBuf>,
    master_password_option: Option<String>,
    keyfile_option: Option<PathBuf>,
    passwords: Vec<Password>,
) -> (bool, String) {
    if let Some(database_path) = database_path {
        let mut db = Database::new(Default::default());
        db.meta.database_name = Some("Passwords Database".to_string());
        passwords.into_iter().for_each(|password| {
            let mut entry = Entry::new();
            entry
                .fields
                .insert("Title".to_string(), Value::Unprotected(password.title));
            entry
                .fields
                .insert("URL".to_string(), Value::Unprotected(password.url));
            entry.fields.insert(
                "UserName".to_string(),
                Value::Unprotected(password.username),
            );
            entry.fields.insert(
                "Password".to_string(),
                Value::Protected(password.password.as_bytes().into()),
            );
            db.root.add_child(entry);
        });
        let mut key = DatabaseKey::new();
        if let Some(master_password) = master_password_option {
            key = key.with_password(&master_password);
        };
        if let Some(keyfile) = keyfile_option {
            match (|| key.with_keyfile(&mut File::open(keyfile)?))() {
                Ok(keyfile_unwrapped) => {
                    key = keyfile_unwrapped;
                }
                Err(err) => {
                    return (
                        false,
                        format!("Failed to save database due to issue with keyfile: {err}"),
                    );
                }
            }
        }
        if let Ok(mut file_handle) = File::create(&database_path) {
            if let Err(error) = db.save(&mut file_handle, key) {
                (false, format!("Failed to save database: {error}"))
            } else {
                (
                    true,
                    format!("Successfully saved database to {database_path:?}"),
                )
            }
        } else {
            (false, String::from("Couldn't open file at database path"))
        }
    } else {
        (false, String::from("Database path was None"))
    }
}

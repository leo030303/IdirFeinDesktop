use keepass::{
    db::{Entry, NodeRef, Value},
    Database, DatabaseKey,
};
use std::{fs::File, path::PathBuf};

use crate::pages::passwords::Password;

pub fn get_passwords(
    keepass_file_path: PathBuf,
    master_password_attempt: Option<&str>,
    keyfile_option: Option<PathBuf>,
) -> Option<Vec<Password>> {
    let mut file = File::open(keepass_file_path).unwrap();
    let mut key = DatabaseKey::new();
    if let Some(master_password) = master_password_attempt {
        key = key.with_password(master_password);
    };
    if let Some(keyfile) = keyfile_option {
        key = key.with_keyfile(&mut File::open(keyfile).unwrap()).unwrap();
    }
    if let Ok(db) = Database::open(&mut file, key) {
        Some(
            db.root
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
                .collect::<Vec<Password>>(),
        )
    } else {
        println!("Error with opening");
        None
    }
}

pub fn save_database(
    database_path: PathBuf,
    master_password_option: Option<&str>,
    keyfile_option: Option<PathBuf>,
    passwords: Vec<Password>,
) {
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
        key = key.with_password(master_password);
    };
    if let Some(keyfile) = keyfile_option {
        key = key.with_keyfile(&mut File::open(keyfile).unwrap()).unwrap();
    }

    db.save(&mut File::create(database_path).unwrap(), key)
        .unwrap();
}

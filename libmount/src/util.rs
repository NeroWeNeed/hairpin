use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

pub fn get_utab_path<'a>() -> Cow<'a, Path> {
    match std::env::var("LIBMOUNT_UTAB") {
        Ok(value) => PathBuf::from(&value).into(),
        Err(err) => match err {
            std::env::VarError::NotPresent => Path::new("/run/mount/utab").into(),
            std::env::VarError::NotUnicode(os_string) => PathBuf::from(&os_string).into(),
        },
    }
}
pub fn get_mtab_path<'a>() -> Cow<'a, Path> {
    match std::env::var("LIBMOUNT_MTAB") {
        Ok(value) => PathBuf::from(&value).into(),
        Err(err) => match err {
            std::env::VarError::NotPresent => Path::new("/etc/mtab").into(),
            std::env::VarError::NotUnicode(os_string) => PathBuf::from(&os_string).into(),
        },
    }
}
pub fn get_fstab_path<'a>() -> Cow<'a, Path> {
    match std::env::var("LIBMOUNT_FSTAB") {
        Ok(value) => PathBuf::from(&value).into(),
        Err(err) => match err {
            std::env::VarError::NotPresent => Path::new("/etc/fstab").into(),
            std::env::VarError::NotUnicode(os_string) => PathBuf::from(&os_string).into(),
        },
    }
}

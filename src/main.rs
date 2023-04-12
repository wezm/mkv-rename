use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use matroska::{Matroska, TagValue};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

fn main() -> ExitCode {
    let flags = xflags::parse_or_exit! {
        /// Don't rename files, just print what would be done
        optional -n,--dry-run
        /// Files to process
        repeated paths: PathBuf
    };

    let mut ok = true;
    for f in flags.paths {
        let path = Path::new(&f);
        match process(path, flags.dry_run) {
            Ok(()) => (),
            Err(err) => {
                eprintln!("Error processing {}: {}", path.display(), err);
                ok = false;
            }
        }
    }

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn process(path: &Path, dry_run: bool) -> Result<(), String> {
    let mkv = matroska::open(path).map_err(|err| err.to_string())?;
    let datetime =
        creation_date(&mkv).ok_or_else(|| String::from("unable to determine creation date"))?;

    // Generate new path
    let new_path = generate_new_path(path, datetime);
    if dry_run {
        println!("{} -> {}", path.display(), new_path.display());
    } else {
        fs::rename(path, &new_path)
            .map_err(|err| format!("unable to rename to {}: {}", new_path.display(), err))?;
    }

    Ok(())
}

fn creation_date(mkv: &Matroska) -> Option<OffsetDateTime> {
    quicktime_creation_date(mkv).or(mkv.info.date_utc)
}

fn quicktime_creation_date(mkv: &Matroska) -> Option<OffsetDateTime> {
    mkv.tags.iter().find_map(|tag| {
        tag.simple
            .iter()
            .find(|simple| simple.name.to_ascii_lowercase() == "com.apple.quicktime.creationdate")
            .and_then(|tag| {
                tag.value.as_ref().and_then(|val| match val {
                    TagValue::String(ref s) => OffsetDateTime::parse(s, &Iso8601::DEFAULT).ok(),
                    TagValue::Binary(_) => None,
                })
            })
    })
}

fn generate_new_path(path: &Path, creation_date: OffsetDateTime) -> PathBuf {
    // prepend a timestamp to the file
    let mut file_name = OsString::from(creation_date.unix_timestamp().to_string());
    file_name.push(" ");
    file_name.push(path.file_name().unwrap()); // file_name should exist at this point
    path.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_new_path() {
        let path = Path::new("folder/IMG_4792.mkv");
        let datetime = OffsetDateTime::from_unix_timestamp(1681265941).unwrap();
        let new_path = generate_new_path(path, datetime);
        let expected_path = Path::new("folder/1681265941 IMG_4792.mkv");
        assert_eq!(new_path, expected_path);
    }
}

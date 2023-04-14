use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use matroska::{Matroska, TagValue};
use mp4::Mp4Reader;
use time::format_description::well_known::{Iso8601, Rfc2822};
use time::{Duration, OffsetDateTime};

struct Flags {
    dry_run: bool,
    /// Offset in seconds
    offset: Duration,
}

fn main() -> ExitCode {
    let all_flags = xflags::parse_or_exit! {
        /// Don't rename files, just print what would be done
        optional -n,--dry-run
        /// Offset in hours (can be fractional) to add to timestamps read from file
        ///
        /// Some cameras appear to store the creation date in local time, without a timezone.
        /// This flag allows those times to be adjusted.
        optional -t,--tz-offset offset: f32
        /// Files to process
        repeated paths: PathBuf
    };
    // TODO: use xflags::xflags! macro and make this a TryFrom impl
    // TODO: use let-else when CI/Alpine is on a new enough Rust version
    // let Some(offset) = f32_to_i32((all_flags.tz_offset.unwrap_or_default() * 60. * 60.).round()) else {
    //     eprintln!("offset too big");
    //     return ExitCode::FAILURE;
    // };
    let offset = match f32_to_i32((all_flags.tz_offset.unwrap_or_default() * 60. * 60.).round()) {
        Some(offset) => offset,
        None => {
            eprintln!("offset too big");
            return ExitCode::FAILURE;
        }
    };
    let flags = Flags {
        dry_run: all_flags.dry_run,
        offset: Duration::new(i64::from(offset), 0),
    };

    let mut ok = true;
    for f in all_flags.paths {
        let path = Path::new(&f);
        match process(path, &flags) {
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

fn process(path: &Path, flags: &Flags) -> Result<(), String> {
    match path
        .extension()
        .map(|ext| ext.to_string_lossy().to_ascii_lowercase())
        .as_deref()
    {
        Some("mkv") => process_matroska(path, flags),
        Some("mov" | "mp4" | "m4v") => process_mp4(path, flags),
        _ => Err(String::from("unknown file type")),
    }
}

fn process_matroska(path: &Path, flags: &Flags) -> Result<(), String> {
    let mkv = matroska::open(path).map_err(|err| err.to_string())?;
    let datetime = mkv_creation_date(&mkv)
        .ok_or_else(|| String::from("unable to determine creation date"))?
        + flags.offset;

    let new_path = generate_new_path(path, datetime);
    println!(
        "{} -> {} ({})",
        path.display(),
        new_path.display(),
        datetime.format(&Rfc2822).unwrap()
    );
    maybe_do_rename(path, &new_path, flags.dry_run)?;

    Ok(())
}

fn process_mp4(path: &Path, flags: &Flags) -> Result<(), String> {
    let f = File::open(path).map_err(|err| err.to_string())?;
    let size = f.metadata().map_err(|err| err.to_string())?.len();
    let reader = BufReader::new(f);
    let mp4 = Mp4Reader::read_header(reader, size).map_err(|err| err.to_string())?;
    let datetime = mp4_creation_date(&mp4)
        .ok_or_else(|| String::from("unable to determine creation date"))?
        + flags.offset;

    let new_path = generate_new_path(path, datetime);
    println!(
        "{} -> {} ({})",
        path.display(),
        new_path.display(),
        datetime.format(&Rfc2822).unwrap()
    );
    maybe_do_rename(path, &new_path, flags.dry_run)?;

    Ok(())
}

fn maybe_do_rename(path: &Path, new_path: &PathBuf, dry_run: bool) -> Result<(), String> {
    if !dry_run {
        fs::rename(path, &new_path)
            .map_err(|err| format!("unable to rename to {}: {}", new_path.display(), err))?;
    }
    Ok(())
}

fn mp4_creation_date<R>(mp4: &Mp4Reader<R>) -> Option<OffsetDateTime> {
    let creation_time = mp4.moov.mvhd.creation_time;

    // convert from MP4 epoch (1904-01-01) to Unix epoch (1970-01-01)
    let timestamp = creation_time as i64 - 2082844800;
    OffsetDateTime::from_unix_timestamp(timestamp).ok()
}

fn mkv_creation_date(mkv: &Matroska) -> Option<OffsetDateTime> {
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

fn f32_to_i32(x: f32) -> Option<i32> {
    (x == (x as i32) as f32).then(|| x as i32)
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

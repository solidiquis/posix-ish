use crate::arg::ProgramStart;
use posix_ish_utils::datetime::to_local_dt;

/// 365 / 2 in seconds
const ALT_DT_THRESHOLD_SECS: i64 = 15_778_800;

/// See the [STDOUT](https://pubs.opengroup.org/onlinepubs/009695099/utilities/ls.html) section.
///
/// If the file has been modified within the past 6 months then the format should take the following
/// form: `date "+%b %e %H:%M"`. Otherwise it should be `date "+%b %e  %Y"` (note two spaces between
/// `%e and `%Y`).
pub fn mtime(mtime: i64, program_start: ProgramStart) -> String {
    let Ok(local) = to_local_dt(mtime) else {
        return format!("{:<12}", "-");
    };
    let libc::tm {
        tm_mon,
        tm_mday,
        tm_hour,
        tm_min,
        tm_year,
        ..
    } = local;

    let mon = match tm_mon {
        0 => "Jan",
        1 => "Feb",
        2 => "Mar",
        3 => "Apr",
        4 => "May",
        5 => "Jun",
        6 => "Jul",
        7 => "Aug",
        8 => "Sep",
        9 => "Oct",
        10 => "Nov",
        11 => "Dec",
        _ => return format!("{:<12}", "-"),
    };

    let show_year = match program_start.checked_sub(mtime) {
        Some(elapsed) => elapsed >= ALT_DT_THRESHOLD_SECS,
        None => true,
    };

    if show_year {
        // https://pubs.opengroup.org/onlinepubs/7908799/xsh/time.h.html
        let year = 1900 + tm_year;
        format!("{mon} {tm_mday:>2}  {year}")
    } else {
        format!("{mon} {tm_mday:>2} {tm_hour:>02}:{tm_min:>02}")
    }
}

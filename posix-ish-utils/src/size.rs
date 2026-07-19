use std::env;

/// `512 B`
pub const POSIX_BLKSIZE: u64 = 512;

/// https://man7.org/linux/man-pages/man1/du.1.html
/// https://en.wikipedia.org/wiki/POSIX
pub fn block_size() -> u64 {
    if env::var_os("POSIXLY_CORRECT").is_some() {
        return POSIX_BLKSIZE;
    } else if env::var_os("POSIX_ME_HARDER").is_some() {
        return POSIX_BLKSIZE;
    }
    env::var("DU_BLOCK_SIZE")
        .ok()
        .or_else(|| env::var("BLOCK_SIZE").ok())
        .or_else(|| env::var("BLOCKSIZE").ok())
        .and_then(|size| size.parse::<u64>().ok())
        .unwrap_or(POSIX_BLKSIZE)
}

pub fn human_bin(blocks: u64, blksize: u64) -> String {
    let bytes = blocks * blksize;
    let exp = bytes.ilog2();
    let bytes_f = bytes as f64;
    let quotient_floor = exp / 10;

    match quotient_floor {
        0 => format!("{bytes} B"),
        1 => format!("{:.2} KiB", bytes_f / (1_u64 << 10) as f64),
        2 => format!("{:.2} MiB", bytes_f / (1_u64 << 20) as f64),
        3 => format!("{:.2} GiB", bytes_f / (1_u64 << 30) as f64),
        4 => format!("{:.2} TiB", bytes_f / (1_u64 << 40) as f64),
        _ => format!("{:.2} PiB", bytes_f / (1_u64 << 50) as f64),
    }
}

pub fn human_si(blocks: u64, blksize: u64) -> String {
    let bytes = blocks * blksize;
    let exp = bytes.ilog10();
    let bytes_f = bytes as f64;
    let quotient_floor = exp / 3;

    match quotient_floor {
        0 => format!("{bytes} B"),
        1 => format!("{:.2} KB", bytes_f / (10_u64.pow(3) as f64)),
        2 => format!("{:.2} MB", bytes_f / (10_u64.pow(6) as f64)),
        3 => format!("{:.2} GB", bytes_f / (10_u64.pow(9) as f64)),
        4 => format!("{:.2} TB", bytes_f / (10_u64.pow(12) as f64)),
        _ => format!("{:.2} PB", bytes_f / (10_u64.pow(15) as f64)),
    }
}

#[test]
fn test_human_bin() {
    assert_eq!(human_bin(1, 512), String::from("512 B"),);
    assert_eq!(human_bin(2, 512), String::from("1.00 KiB"),);
    assert_eq!(human_bin(1, 1024), String::from("1.00 KiB"),);
    assert_eq!(human_bin(1024, 1024), String::from("1.00 MiB"),);
    assert_eq!(human_bin(1124, 1024), String::from("1.10 MiB"),);
    assert_eq!(human_bin(1024 * 1024, 1024), String::from("1.00 GiB"),);
    assert_eq!(
        human_bin(1024 * 1024 * 1024, 1024),
        String::from("1.00 TiB"),
    );
    assert_eq!(
        human_bin(1024 * 1024 * 1024 * 1024, 1024),
        String::from("1.00 PiB"),
    );
    assert_eq!(
        human_bin(1024 * 1024 * 1024 * 1024 * 1024, 1024),
        String::from("1024.00 PiB"),
    );
}

#[test]
fn test_human_si() {
    assert_eq!(human_si(1, 500), String::from("500 B"),);
    assert_eq!(human_si(2, 512), String::from("1.02 KB"),);
    assert_eq!(human_si(1, 1000), String::from("1.00 KB"),);
    assert_eq!(human_si(1000, 1000), String::from("1.00 MB"),);
    assert_eq!(human_si(1000 * 1000, 1000), String::from("1.00 GB"),);
    assert_eq!(human_si(1000 * 1000 * 1000, 1000), String::from("1.00 TB"),);
    assert_eq!(
        human_si(1000 * 1000 * 1000 * 1000, 1000),
        String::from("1.00 PB"),
    );
    assert_eq!(
        human_si(1000 * 1000 * 1000 * 1000 * 1000, 1000),
        String::from("1000.00 PB"),
    );
}

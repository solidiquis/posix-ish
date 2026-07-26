use std::{
    env,
    fmt::{self, Display},
};

/// `512 B`
pub const POSIX_BLKSIZE: u64 = 512;

#[derive(Copy, Clone)]
pub enum Bi {
    Bytes(u64),
    Kibi(f64),
    Mebi(f64),
    Gibi(f64),
    Tebi(f64),
    Pebi(f64),
}

#[derive(Copy, Clone)]
pub enum Si {
    Bytes(u64),
    Kilo(f64),
    Mega(f64),
    Giga(f64),
    Terra(f64),
    Peta(f64),
}

pub struct BiDisplay {
    inner: Bi,
    compact: bool,
}

pub struct SiDisplay {
    inner: Si,
    compact: bool,
}

/// https://man7.org/linux/man-pages/man1/du.1.html
/// https://en.wikipedia.org/wiki/POSIX
pub fn block_size() -> u64 {
    if env::var_os("POSIXLY_CORRECT").is_some() || env::var_os("POSIX_ME_HARDER").is_some() {
        return POSIX_BLKSIZE;
    }
    env::var("DU_BLOCK_SIZE")
        .ok()
        .or_else(|| env::var("BLOCK_SIZE").ok())
        .or_else(|| env::var("BLOCKSIZE").ok())
        .and_then(|size| size.parse::<u64>().ok())
        .unwrap_or(POSIX_BLKSIZE)
}

pub fn human_bin(blocks: u64, blksize: u64) -> Bi {
    let bytes = blocks * blksize;
    apparent_human_bin(bytes)
}

pub fn apparent_human_bin(bytes: u64) -> Bi {
    let exp = bytes.checked_ilog2().unwrap_or_default();
    let bytes_f = bytes as f64;
    let quotient_floor = exp / 10;

    match quotient_floor {
        0 => Bi::Bytes(bytes),
        1 => Bi::Kibi(bytes_f / (1_u64 << 10) as f64),
        2 => Bi::Mebi(bytes_f / (1_u64 << 20) as f64),
        3 => Bi::Gibi(bytes_f / (1_u64 << 30) as f64),
        4 => Bi::Tebi(bytes_f / (1_u64 << 40) as f64),
        _ => Bi::Pebi(bytes_f / (1_u64 << 50) as f64),
    }
}

pub fn human_si(blocks: u64, blksize: u64) -> Si {
    let bytes = blocks * blksize;
    apparent_human_si(bytes)
}

pub fn apparent_human_si(bytes: u64) -> Si {
    let exp = bytes.checked_ilog10().unwrap_or_default();
    let bytes_f = bytes as f64;
    let quotient_floor = exp / 3;

    match quotient_floor {
        0 => Si::Bytes(bytes),
        1 => Si::Kilo(bytes_f / (10_u64.pow(3) as f64)),
        2 => Si::Mega(bytes_f / (10_u64.pow(6) as f64)),
        3 => Si::Giga(bytes_f / (10_u64.pow(9) as f64)),
        4 => Si::Terra(bytes_f / (10_u64.pow(12) as f64)),
        _ => Si::Peta(bytes_f / (10_u64.pow(15) as f64)),
    }
}

impl Bi {
    pub fn display(&self, compact: bool) -> BiDisplay {
        BiDisplay {
            compact,
            inner: *self,
        }
    }
}

impl Si {
    pub fn display(&self, compact: bool) -> SiDisplay {
        SiDisplay {
            compact,
            inner: *self,
        }
    }
}

impl Display for BiDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.compact {
            match self.inner {
                Bi::Bytes(n) => write!(f, "{n}B"),
                Bi::Kibi(n) => write!(f, "{n:.0}Ki"),
                Bi::Mebi(n) => write!(f, "{n:.0}Mi"),
                Bi::Gibi(n) => write!(f, "{n:.0}Gi"),
                Bi::Tebi(n) => write!(f, "{n:.0}Ti"),
                Bi::Pebi(n) => write!(f, "{n:.0}Pi"),
            }
        } else {
            match self.inner {
                Bi::Bytes(n) => write!(f, "{n} B"),
                Bi::Kibi(n) => write!(f, "{n:.2} KiB"),
                Bi::Mebi(n) => write!(f, "{n:.2} MiB"),
                Bi::Gibi(n) => write!(f, "{n:.2} GiB"),
                Bi::Tebi(n) => write!(f, "{n:.2} TiB"),
                Bi::Pebi(n) => write!(f, "{n:.2} PiB"),
            }
        }
    }
}

impl Display for SiDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.compact {
            match self.inner {
                Si::Bytes(n) => write!(f, "{n}B"),
                Si::Kilo(n) => write!(f, "{n:.0}K"),
                Si::Mega(n) => write!(f, "{n:.0}M"),
                Si::Giga(n) => write!(f, "{n:.0}G"),
                Si::Terra(n) => write!(f, "{n:.0}T"),
                Si::Peta(n) => write!(f, "{n:.0}P"),
            }
        } else {
            match self.inner {
                Si::Bytes(n) => write!(f, "{n} B"),
                Si::Kilo(n) => write!(f, "{n:.2} KB"),
                Si::Mega(n) => write!(f, "{n:.2} MB"),
                Si::Giga(n) => write!(f, "{n:.2} GB"),
                Si::Terra(n) => write!(f, "{n:.2} TB"),
                Si::Peta(n) => write!(f, "{n:.2} PB"),
            }
        }
    }
}

#[test]
fn test_human_bin() {
    assert_eq!(
        format!("{}", human_bin(1, 512).display(false)),
        String::from("512 B"),
    );
    assert_eq!(
        format!("{}", human_bin(2, 512).display(false)),
        String::from("1.00 KiB"),
    );
    assert_eq!(
        format!("{}", human_bin(1, 1024).display(false)),
        String::from("1.00 KiB"),
    );
    assert_eq!(
        format!("{}", human_bin(1024, 1024).display(false)),
        String::from("1.00 MiB"),
    );
    assert_eq!(
        format!("{}", human_bin(1124, 1024).display(false)),
        String::from("1.10 MiB"),
    );
    assert_eq!(
        format!("{}", human_bin(1024 * 1024, 1024).display(false)),
        String::from("1.00 GiB"),
    );
    assert_eq!(
        format!("{}", human_bin(1024 * 1024 * 1024, 1024).display(false)),
        String::from("1.00 TiB"),
    );
    assert_eq!(
        format!(
            "{}",
            human_bin(1024 * 1024 * 1024 * 1024, 1024).display(false)
        ),
        String::from("1.00 PiB"),
    );
    assert_eq!(
        format!(
            "{}",
            human_bin(1024 * 1024 * 1024 * 1024 * 1024, 1024).display(false)
        ),
        String::from("1024.00 PiB"),
    );
}

#[test]
fn test_human_si() {
    assert_eq!(
        format!("{}", human_si(1, 500).display(false)),
        String::from("500 B")
    );
    assert_eq!(
        format!("{}", human_si(2, 512).display(false)),
        String::from("1.02 KB")
    );
    assert_eq!(
        format!("{}", human_si(1, 1000).display(false)),
        String::from("1.00 KB")
    );
    assert_eq!(
        format!("{}", human_si(1000, 1000).display(false)),
        String::from("1.00 MB")
    );
    assert_eq!(
        format!("{}", human_si(1000 * 1000, 1000).display(false)),
        String::from("1.00 GB")
    );
    assert_eq!(
        format!("{}", human_si(1000 * 1000 * 1000, 1000).display(false)),
        String::from("1.00 TB")
    );
    assert_eq!(
        format!(
            "{}",
            human_si(1000 * 1000 * 1000 * 1000, 1000).display(false)
        ),
        String::from("1.00 PB"),
    );
    assert_eq!(
        format!(
            "{}",
            human_si(1000 * 1000 * 1000 * 1000 * 1000, 1000).display(false)
        ),
        String::from("1000.00 PB"),
    );
}

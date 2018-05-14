#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::*;

pub enum PathWritability {
    Writable,
    NotWritable,
    NotExist,
}

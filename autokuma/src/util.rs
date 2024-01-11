pub use kuma_client::util::ResultLogger;
use std::collections::BTreeMap;

pub fn group_by_prefix<A, B, I>(v: I, delimiter: &str) -> BTreeMap<String, Vec<(String, String)>>
where
    A: AsRef<str>,
    B: AsRef<str>,
    I: IntoIterator<Item = (A, B)>,
{
    v.into_iter()
        .fold(BTreeMap::new(), |mut groups, (key, value)| {
            if let Some((prefix, key)) = key.as_ref().split_once(delimiter) {
                groups
                    .entry(prefix.to_owned())
                    .or_default()
                    .push((key.to_owned(), value.as_ref().to_owned()));
            }
            groups
        })
}

pub trait ResultOrDie<T> {
    fn unwrap_or_die(self, exit_code: i32) -> T;
}

impl<T, E> ResultOrDie<T> for std::result::Result<T, E> {
    fn unwrap_or_die(self, exit_code: i32) -> T {
        match self {
            Ok(t) => t,
            Err(_) => std::process::exit(exit_code),
        }
    }
}

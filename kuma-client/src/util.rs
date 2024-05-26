use log::{debug, error, info, trace, warn};

pub trait ResultLogger<F> {
    fn log_trace(self, target: &str, cb: F) -> Self;
    fn log_debug(self, target: &str, cb: F) -> Self;
    fn log_info(self, target: &str, cb: F) -> Self;
    fn log_warn(self, target: &str, cb: F) -> Self;
    fn log_error(self, target: &str, cb: F) -> Self;
    fn print_error(self, cb: F) -> Self;
}

impl<F, S, T, E> ResultLogger<F> for std::result::Result<T, E>
where
    S: AsRef<str>,
    F: FnOnce(&E) -> S,
{
    fn log_trace(self, target: &str, cb: F) -> Self {
        return self.map_err(|e| {
            trace!(target: target, "{}", cb(&e).as_ref());
            e
        });
    }

    fn log_debug(self, target: &str, cb: F) -> Self {
        return self.map_err(|e| {
            debug!(target: target, "{}", cb(&e).as_ref());
            e
        });
    }

    fn log_info(self, target: &str, cb: F) -> Self {
        return self.map_err(|e| {
            info!(target: target, "{}", cb(&e).as_ref());
            e
        });
    }

    fn log_warn(self, target: &str, cb: F) -> Self {
        return self.map_err(|e| {
            warn!(target: target, "{}", cb(&e).as_ref());
            e
        });
    }

    fn log_error(self, target: &str, cb: F) -> Self {
        return self.map_err(|e| {
            error!(target: target, "{}", cb(&e).as_ref());
            e
        });
    }

    fn print_error(self, cb: F) -> Self {
        return self.map_err(|e| {
            println!("{}", cb(&e).as_ref());
            e
        });
    }
}

impl<F, S, T> ResultLogger<F> for Option<T>
where
    S: AsRef<str>,
    F: FnOnce() -> S,
{
    fn log_trace(self, target: &str, cb: F) -> Self {
        if self.is_none() {
            trace!(target: target, "{}", cb().as_ref())
        }
        self
    }

    fn log_debug(self, target: &str, cb: F) -> Self {
        if self.is_none() {
            debug!(target: target, "{}", cb().as_ref())
        }
        self
    }

    fn log_info(self, target: &str, cb: F) -> Self {
        if self.is_none() {
            info!(target: target, "{}", cb().as_ref())
        }
        self
    }

    fn log_warn(self, target: &str, cb: F) -> Self {
        if self.is_none() {
            warn!(target: target, "{}", cb().as_ref())
        }
        self
    }

    fn log_error(self, target: &str, cb: F) -> Self {
        if self.is_none() {
            error!(target: target, "{}", cb().as_ref())
        }
        self
    }

    fn print_error(self, cb: F) -> Self {
        if self.is_none() {
            println!("{}", cb().as_ref())
        }
        self
    }
}

#[macro_export]
macro_rules! default_from_serde {
    ($struct_name:ident) => {
        impl Default for $struct_name {
            fn default() -> Self {
                serde_json::from_value(serde_json::json!({})).unwrap()
            }
        }

        impl $struct_name {
            pub fn new() -> Self {
                Default::default()
            }
        }
    };
}

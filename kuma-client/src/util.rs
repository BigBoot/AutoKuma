use log::{debug, error, info, trace, warn};

pub trait ResultLogger<F> {
    fn log_trace(self, cb: F) -> Self;
    fn log_debug(self, cb: F) -> Self;
    fn log_info(self, cb: F) -> Self;
    fn log_warn(self, cb: F) -> Self;
    fn log_error(self, cb: F) -> Self;
}

impl<F, S, T, E> ResultLogger<F> for std::result::Result<T, E>
where
    S: AsRef<str>,
    F: FnOnce(&E) -> S,
{
    fn log_trace(self, cb: F) -> Self {
        return self.map_err(|e| {
            trace!("{}", cb(&e).as_ref());
            e
        });
    }

    fn log_debug(self, cb: F) -> Self {
        return self.map_err(|e| {
            debug!("{}", cb(&e).as_ref());
            e
        });
    }

    fn log_info(self, cb: F) -> Self {
        return self.map_err(|e| {
            info!("{}", cb(&e).as_ref());
            e
        });
    }

    fn log_warn(self, cb: F) -> Self {
        return self.map_err(|e| {
            warn!("{}", cb(&e).as_ref());
            e
        });
    }

    fn log_error(self, cb: F) -> Self {
        return self.map_err(|e| {
            error!("{}", cb(&e).as_ref());
            e
        });
    }
}

impl<F, S, T> ResultLogger<F> for Option<T>
where
    S: AsRef<str>,
    F: FnOnce() -> S,
{
    fn log_trace(self, cb: F) -> Self {
        if self.is_none() {
            trace!("{}", cb().as_ref())
        }
        self
    }

    fn log_debug(self, cb: F) -> Self {
        if self.is_none() {
            debug!("{}", cb().as_ref())
        }
        self
    }

    fn log_info(self, cb: F) -> Self {
        if self.is_none() {
            info!("{}", cb().as_ref())
        }
        self
    }

    fn log_warn(self, cb: F) -> Self {
        if self.is_none() {
            warn!("{}", cb().as_ref())
        }
        self
    }

    fn log_error(self, cb: F) -> Self {
        if self.is_none() {
            error!("{}", cb().as_ref())
        }
        self
    }
}

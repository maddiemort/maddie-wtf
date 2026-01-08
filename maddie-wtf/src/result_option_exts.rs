pub trait ResultExt<T, E> {
    type Swapped<T2, E2>;
    fn swap(self) -> Self::Swapped<E, T>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    type Swapped<T2, E2> = Result<T2, E2>;

    fn swap(self) -> Self::Swapped<E, T> {
        match self {
            Ok(t) => Err(t),
            Err(e) => Ok(e),
        }
    }
}

pub trait OptionExt<T> {
    fn err_or<T2>(self, ok: T2) -> Result<T2, T>;
    fn err_or_else<T2, F>(self, ok: F) -> Result<T2, T>
    where
        F: FnOnce() -> T2;
}

impl<T> OptionExt<T> for Option<T> {
    fn err_or<T2>(self, ok: T2) -> Result<T2, T> {
        match self {
            Some(v) => Err(v),
            None => Ok(ok),
        }
    }

    fn err_or_else<T2, F>(self, ok: F) -> Result<T2, T>
    where
        F: FnOnce() -> T2,
    {
        match self {
            Some(v) => Err(v),
            None => Ok(ok()),
        }
    }
}

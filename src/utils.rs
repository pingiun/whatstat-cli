macro_rules! printerr {
    ($($arg:tt)*) => (write!($crate::std::io::stderr(), $($arg)*));
}

macro_rules! printerrln {
    ($fmt:expr) => (printerr!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (printerr!(concat!($fmt, "\n"), $($arg)*));
}

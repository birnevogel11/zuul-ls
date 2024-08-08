#[macro_export]
macro_rules! safe_println {
    ( $( $t:tt )* ) => {
         let _ = match calm_io::stdoutln!($( $t )*) {
            Ok(_) => Ok(()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::BrokenPipe => Ok(()),
                _ => Err(e),
            },
        };
    };
}

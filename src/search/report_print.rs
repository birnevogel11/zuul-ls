use crate::parser::common::StringLoc;
use crate::search::job_vars::VariableInfo;
use crate::search::path::shorten_path;

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

pub fn print_var_info_list(vars: Vec<VariableInfo>) {
    for var_info in vars {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            var_info.name.value,
            var_info.job_name.value,
            var_info.value,
            shorten_path(&var_info.name.path).display(),
            var_info.name.line,
            var_info.name.col,
        )
    }
}

pub fn print_string_locs(locs: &[StringLoc]) {
    for loc in locs {
        safe_println!(
            "{}\t{}\t{}\t{}",
            loc.value,
            shorten_path(&loc.path).display(),
            loc.line,
            loc.col
        );
    }
}

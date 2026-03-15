// Wrapper binary: ypdomainname -> hostname -y
fn main() {
    uucore::panic::mute_sigpipe_panic();

    use std::ffi::OsString;
    use std::process;

    let args = std::env::args_os().skip(1);
    let iter = (vec![OsString::from("hostname"), OsString::from("-y")].into_iter()).chain(args);

    process::exit(hostname::uumain(iter));
}

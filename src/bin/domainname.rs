// Wrapper binary: domainname -> hostname -d
fn main() {
    uucore::panic::mute_sigpipe_panic();

    use std::ffi::OsString;
    use std::process;

    let args = std::env::args_os().skip(1);
    let iter = (vec![OsString::from("hostname"), OsString::from("-d")].into_iter()).chain(args);

    process::exit(hostname::uumain(iter));
}

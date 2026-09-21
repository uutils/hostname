// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;
use std::ffi::OsString;

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    let exit_code = domainname::uumain(args.into_iter());
    std::process::exit(exit_code);
}

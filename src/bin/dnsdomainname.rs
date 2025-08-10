// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let exit_code = uu_dnsdomainname::uumain(args);
    std::process::exit(match exit_code {
        Ok(()) => 0,
        Err(_) => 1,
    });
}

// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(not(target_family = "windows"))]
mod unix;

#[cfg(not(target_family = "windows"))]
pub(crate) use unix::print_domain_name;

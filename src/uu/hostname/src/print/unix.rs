// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::{CStr, c_int, c_uint};
use std::ptr::NonNull;

use libc::{
    AF_UNSPEC, AI_CANONNAME, IFF_LOOPBACK, IFF_UP, NI_NAMEREQD, NI_NUMERICHOST, SOCK_DGRAM,
    sockaddr, sockaddr_in, sockaddr_in6, socklen_t,
};
use uucore::error::UResult;

use crate::errors::{GetNameOrAddrInfoError, HostNameError};
use crate::net::{
    AddressInfo, InterfaceAddresses, domain_name, get_name_info, host_name, in6_is_addr_linklocal,
    in6_is_addr_mc_linklocal, short_host_name,
};
use crate::print::{
    AliasHostName, AllFqdnHostName, AllIpAddressesHostName, DefaultHostName, DomainHostName,
    FqdnHostName, IpAddressHostName, NisHostName, PrintHostName, ShortHostName,
};

impl PrintHostName for DefaultHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let host_name = host_name()?;
        out.write_all(host_name.as_bytes())?;
        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for AliasHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        // This is intended to reproduce the behavior of calling gethostbyname() and then printing
        // the elements of hostent::h_aliases[].
        //
        // However, gethostbyname() is deprecated. Replacing its usage with getaddrinfo()
        // and getnameinfo() does NOT produce the same result, but it is the best approach I can
        // think of, that is still portable.

        let host_name = host_name()?;
        let address_info = AddressInfo::new(&host_name, AF_UNSPEC, SOCK_DGRAM, 0, 0)?;

        out.write_all(host_name.as_bytes())?;

        address_info
            .iter()
            .map(|ai| get_name_info(ai.ai_addr, ai.ai_addrlen, NI_NAMEREQD))
            .try_for_each(|name| -> UResult<()> {
                out.write_all(b" ")?;
                out.write_all(name?.as_bytes()).map_err(From::from)
            })?;

        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for DomainHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let address_info = AddressInfo::new(&host_name()?, AF_UNSPEC, SOCK_DGRAM, 0, AI_CANONNAME)?;

        let canonical_name = address_info.first().ai_canonname;
        if canonical_name.is_null() {
            return Ok(()); // No canonical name set.
        };

        let Some(domain_name) = unsafe { CStr::from_ptr(canonical_name) }
            .to_bytes()
            .splitn(2, |&byte| byte == b'.')
            .nth(1)
        else {
            return Ok(()); // Canonical name contains zero dots.
        };

        out.write_all(domain_name)?;
        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for FqdnHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let address_info = AddressInfo::new(&host_name()?, AF_UNSPEC, SOCK_DGRAM, 0, AI_CANONNAME)?;

        let canonical_name = address_info.first().ai_canonname;
        if canonical_name.is_null() {
            return Ok(()); // No canonical name set.
        };

        out.write_all(unsafe { CStr::from_ptr(canonical_name) }.to_bytes())?;
        out.write_all(b"\n").map_err(From::from)
    }
}

#[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
fn filter_map_interface_addresses(
    interface_address: &libc::ifaddrs,
) -> Option<(NonNull<sockaddr>, socklen_t)> {
    // Ensure the interface has a configured address.
    let addr = NonNull::new(interface_address.ifa_addr)?;

    if (interface_address.ifa_flags & (IFF_UP as c_uint)) == 0 {
        return None; // Interface is down.
    }

    if (interface_address.ifa_flags & (IFF_LOOPBACK as c_uint)) != 0 {
        return None; // This is the loop back interface.
    }

    match c_int::from(unsafe { addr.as_ref() }.sa_family) {
        libc::AF_INET => Some((addr, size_of::<sockaddr_in>() as socklen_t)),

        libc::AF_INET6 => {
            let ipv6_addr = unsafe { &addr.cast::<sockaddr_in6>().as_ref().sin6_addr };
            // Ensure ipv6_addr is not an IPv6 link-local address.
            (!in6_is_addr_linklocal(ipv6_addr) && !in6_is_addr_mc_linklocal(ipv6_addr))
                .then_some((addr, size_of::<sockaddr_in6>() as socklen_t))
        }

        _ => None, // Unsupported address family.
    }
}

impl PrintHostName for AllFqdnHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let mut separator: &[u8] = &[];

        InterfaceAddresses::new()?
            .iter()
            .filter_map(filter_map_interface_addresses)
            // Skip addresses whose translation fails.
            .filter_map(|(addr, size)| get_name_info(addr.as_ptr(), size, NI_NAMEREQD).ok())
            .try_for_each(|name| {
                out.write_all(separator)?;
                separator = b" ";
                out.write_all(name.as_bytes())
            })?;

        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for IpAddressHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let mut separator: &[u8] = &[];

        AddressInfo::new(&host_name()?, AF_UNSPEC, SOCK_DGRAM, 0, 0)?
            .iter()
            .map(|ai| get_name_info(ai.ai_addr, ai.ai_addrlen, NI_NUMERICHOST))
            .try_for_each(|name| -> UResult<()> {
                out.write_all(separator)?;
                separator = b" ";
                out.write_all(name?.as_bytes()).map_err(From::from)
            })?;

        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for AllIpAddressesHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        const NONAME: HostNameError =
            HostNameError::GetNameOrAddrInfo(GetNameOrAddrInfoError(libc::EAI_NONAME));

        let mut separator: &[u8] = &[];

        InterfaceAddresses::new()?
            .iter()
            .filter_map(filter_map_interface_addresses)
            .map(|(addr, addr_size)| get_name_info(addr.as_ptr(), addr_size, NI_NUMERICHOST))
            .filter(|result| *result != Err(NONAME))
            .try_for_each(|name| -> UResult<()> {
                out.write_all(separator)?;
                separator = b" ";
                out.write_all(name?.as_bytes()).map_err(From::from)
            })?;

        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for ShortHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let host_name = short_host_name()?;
        out.write_all(host_name.as_bytes())?;
        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for NisHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        if let Some(domain_name) = domain_name()? {
            out.write_all(domain_name.as_bytes())?;
            out.write_all(b"\n").map_err(From::from)
        } else {
            Err(Box::new(HostNameError::NoLocalDomainName))
        }
    }
}

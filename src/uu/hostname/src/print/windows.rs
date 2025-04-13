// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::c_int;

use uucore::error::UResult;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    IF_TYPE_SOFTWARE_LOOPBACK, IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_IPV4_ENABLED,
    IP_ADAPTER_IPV6_ENABLED, IP_ADAPTER_UNICAST_ADDRESS_LH,
};
use windows_sys::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows_sys::Win32::Networking::WinSock::{
    AF_INET, AF_INET6, AF_UNSPEC, NI_NAMEREQD, NI_NUMERICHOST, SOCK_DGRAM, SOCKADDR, SOCKADDR_IN6,
    WSAHOST_NOT_FOUND,
};

use crate::errors::HostNameError;
use crate::net::{
    AdapterUnicastAddressIter, AddressInfo, InterfaceAddresses, domain_name,
    fully_qualified_dns_name, get_name_info, host_name, in6_is_addr_linklocal,
    in6_is_addr_mc_linklocal, short_host_name,
};
use crate::print::{
    AliasHostName, AllFqdnHostName, AllIpAddressesHostName, DefaultHostName, DomainHostName,
    FqdnHostName, IpAddressHostName, NisHostName, PrintHostName, ShortHostName,
};

impl PrintHostName for DefaultHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let host_name = host_name()?;
        out.write_all(host_name.as_encoded_bytes())?;
        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for AliasHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        // This is intended to reproduce the behavior of calling gethostbyname() and then printing
        // the elements of hostent::h_aliases[].
        //
        // However, gethostbyname() is deprecated. Replacing its usage with GetAddrInfoW()
        // and GetNameInfoW() does NOT produce the same result, but it is the best approach I can
        // think of, that is still portable.

        let host_name = host_name()?;
        out.write_all(host_name.as_encoded_bytes())?;

        AddressInfo::new(&host_name, AF_UNSPEC as c_int, SOCK_DGRAM, 0, 0)?
            .iter()
            .map(|ai| get_name_info(ai.ai_addr, ai.ai_addrlen, NI_NAMEREQD as c_int))
            .try_for_each(|name| -> UResult<()> {
                out.write_all(b" ")?;
                out.write_all(name?.as_encoded_bytes()).map_err(From::from)
            })?;

        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for DomainHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        if let Some(domain_name) = domain_name()? {
            out.write_all(domain_name.as_encoded_bytes())?;
            out.write_all(b"\n").map_err(From::from)
        } else {
            Ok(())
        }
    }
}

impl PrintHostName for FqdnHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let host_name = fully_qualified_dns_name()?;
        out.write_all(host_name.as_encoded_bytes())?;
        out.write_all(b"\n").map_err(From::from)
    }
}

fn filter_interface_addresses(ai: &&IP_ADAPTER_ADDRESSES_LH) -> bool {
    // Interface is up.
    ai.OperStatus == IfOperStatusUp &&
    // This is NOT the loop back interface.
    (ai.IfType & IF_TYPE_SOFTWARE_LOOPBACK) == 0 &&
    // Ensure the interface has a configured address.
    !ai.FirstUnicastAddress.is_null() &&
    // Ensure the interface has an IPv4 or IPv6 address.
    // Windows Vista or later is required for this check.
    (unsafe { ai.Anonymous2.Flags } & (IP_ADAPTER_IPV4_ENABLED | IP_ADAPTER_IPV6_ENABLED)) != 0
}

fn filter_map_interface_addresses(
    addr: &IP_ADAPTER_UNICAST_ADDRESS_LH,
) -> Option<(*mut SOCKADDR, usize)> {
    let Ok(addr_size) = usize::try_from(addr.Address.iSockaddrLength) else {
        return None;
    };

    match unsafe { *addr.Address.lpSockaddr }.sa_family {
        AF_INET => Some((addr.Address.lpSockaddr, addr_size)),

        AF_INET6 => {
            let ipv6_addr = unsafe { &(*addr.Address.lpSockaddr.cast::<SOCKADDR_IN6>()).sin6_addr };
            // Ensure ipv6_addr is not an IPv6 link-local address.
            (!in6_is_addr_linklocal(ipv6_addr) && !in6_is_addr_mc_linklocal(ipv6_addr))
                .then_some((addr.Address.lpSockaddr, addr_size))
        }

        _ => None, // Unsupported address family.
    }
}

impl PrintHostName for AllFqdnHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let mut separator: &[u8] = &[];

        InterfaceAddresses::new()?
            .iter()
            .filter(filter_interface_addresses)
            .flat_map(AdapterUnicastAddressIter::new)
            .filter_map(filter_map_interface_addresses)
            // Skip addresses whose translation fails.
            .filter_map(|(addr, size)| get_name_info(addr, size, NI_NAMEREQD as c_int).ok())
            .try_for_each(|name| {
                out.write_all(separator)?;
                separator = b" ";
                out.write_all(name.as_encoded_bytes())
            })?;

        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for IpAddressHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let mut separator: &[u8] = &[];

        AddressInfo::new(&host_name()?, AF_UNSPEC as c_int, SOCK_DGRAM, 0, 0)?
            .iter()
            .map(|ai| get_name_info(ai.ai_addr, ai.ai_addrlen, NI_NUMERICHOST as c_int))
            .try_for_each(|name| -> UResult<()> {
                out.write_all(separator)?;
                separator = b" ";
                out.write_all(name?.as_encoded_bytes()).map_err(From::from)
            })?;

        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for AllIpAddressesHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let mut separator: &[u8] = &[];

        InterfaceAddresses::new()?
            .iter()
            .filter(filter_interface_addresses)
            .flat_map(AdapterUnicastAddressIter::new)
            .filter_map(filter_map_interface_addresses)
            .map(|(addr, size)| get_name_info(addr, size, NI_NUMERICHOST as c_int))
            .filter(|r| !matches!(r, Err(err) if err.raw_os_error() == Some(WSAHOST_NOT_FOUND)))
            .try_for_each(|name| {
                out.write_all(separator)?;
                separator = b" ";
                out.write_all(name?.as_encoded_bytes())
            })?;

        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for ShortHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        let host_name = short_host_name()?;
        out.write_all(host_name.as_encoded_bytes())?;
        out.write_all(b"\n").map_err(From::from)
    }
}

impl PrintHostName for NisHostName {
    fn print_host_name(&self, out: &mut dyn std::io::Write) -> UResult<()> {
        if let Some(domain_name) = domain_name()? {
            out.write_all(domain_name.as_encoded_bytes())?;
            out.write_all(b"\n").map_err(From::from)
        } else {
            Err(Box::new(HostNameError::NoLocalDomainName))
        }
    }
}

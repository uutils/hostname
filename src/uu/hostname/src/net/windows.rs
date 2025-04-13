// This file is part of the uutils hostname package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::ffi::{OsStr, OsString, c_int};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::NonNull;
use std::{mem, ptr};

use uucore::error::UResult;
use windows_sys::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_BUFFER_OVERFLOW, ERROR_INVALID_NAME, ERROR_INVALID_PARAMETER,
    ERROR_MORE_DATA, ERROR_NO_DATA, ERROR_SUCCESS,
};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_FRIENDLY_NAME, GetAdaptersAddresses,
    IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_UNICAST_ADDRESS_LH,
};
use windows_sys::Win32::Networking::WinSock::{
    ADDRINFOW, AF_UNSPEC, FreeAddrInfoW, GetAddrInfoW, GetNameInfoW, IN6_ADDR, SOCKADDR,
    WSACleanup, WSADATA, WSAENAMETOOLONG, WSAENOBUFS, WSAGetLastError, WSAStartup, socklen_t,
};
use windows_sys::Win32::System::SystemInformation::{
    ComputerNamePhysicalDnsDomain, ComputerNamePhysicalDnsFullyQualified,
    ComputerNamePhysicalDnsHostname, GetComputerNameExW, SetComputerNameExW,
};

use crate::errors::HostNameError;

impl crate::net::LibraryGuard {
    pub(crate) fn load() -> std::io::Result<Self> {
        let mut init_data: WSADATA = unsafe { mem::zeroed() };
        let r = unsafe { WSAStartup(make_word(2, 2), &mut init_data) };
        if r == 0 {
            Ok(Self)
        } else {
            Err(std::io::Error::from_raw_os_error(r))
        }
    }
}

impl Drop for crate::net::LibraryGuard {
    fn drop(&mut self) {
        unsafe { WSACleanup() };
    }
}

fn make_word(low: u8, high: u8) -> u16 {
    (u16::from(high) << 8) | u16::from(low)
}

fn in6_is_addr_multicast(addr: &IN6_ADDR) -> bool {
    unsafe { addr.u.Byte[0] == 0xff }
}

pub(crate) fn in6_is_addr_linklocal(addr: &IN6_ADDR) -> bool {
    unsafe { addr.u.Byte[0] == 0xfe && ((addr.u.Byte[1] & 0xc0) == 0x80) }
}

pub(crate) fn in6_is_addr_mc_linklocal(addr: &IN6_ADDR) -> bool {
    in6_is_addr_multicast(addr) && unsafe { (addr.u.Byte[1] & 0xf) == 0x2 }
}

fn get_computer_name_ex_w(kind: c_int) -> std::io::Result<OsString> {
    use std::io::Error;

    let mut buffer: Vec<u16> = Vec::with_capacity(256);
    loop {
        let Ok(mut buffer_capacity) = u32::try_from(buffer.capacity()) else {
            break Err(Error::from_raw_os_error(WSAENAMETOOLONG));
        };

        let buffer_ptr = buffer.spare_capacity_mut().as_mut_ptr().cast();

        if unsafe { GetComputerNameExW(kind, buffer_ptr, &mut buffer_capacity) } == 0 {
            let err = Error::last_os_error();
            if err.raw_os_error() == Some(ERROR_MORE_DATA as i32) {
                // An error happened because a bigger buffer is needed.
                buffer.reserve((buffer_capacity as usize).saturating_sub(buffer.capacity()));
            } else {
                break Err(err);
            }
        } else {
            unsafe { buffer.set_len(buffer_capacity as usize) };
            break Ok(OsString::from_wide(&buffer));
        }
    }
}

pub(crate) fn host_name() -> std::io::Result<OsString> {
    // TODO: Should we specify ComputerNameDnsHostname instead?
    get_computer_name_ex_w(ComputerNamePhysicalDnsHostname)
}

pub(crate) fn fully_qualified_dns_name() -> std::io::Result<OsString> {
    // TODO: Should we specify ComputerNameDnsFullyQualified instead?
    get_computer_name_ex_w(ComputerNamePhysicalDnsFullyQualified)
}

pub(crate) fn domain_name() -> std::io::Result<Option<OsString>> {
    // TODO: Should we specify ComputerNameDnsDomain instead?
    // TODO: Should we call NetGetJoinInformation() instead?
    let name = get_computer_name_ex_w(ComputerNamePhysicalDnsDomain)?;
    Ok((!name.is_empty()).then_some(name))
}

pub(crate) fn short_host_name() -> std::io::Result<OsString> {
    let host_name = host_name()?;
    let bytes = host_name.encode_wide();
    if bytes.clone().any(|ch| ch == (b'.' as u16)) {
        let bytes: Vec<u16> = bytes.take_while(|&ch| ch != (b'.' as u16)).collect();
        Ok(OsString::from_wide(&bytes))
    } else {
        Ok(host_name)
    }
}

pub(crate) fn set_host_name(host_name: &[u16]) -> UResult<()> {
    if unsafe { SetComputerNameExW(ComputerNamePhysicalDnsHostname, host_name.as_ptr()) } != 0 {
        return Ok(());
    }

    let err = std::io::Error::last_os_error();
    match err.raw_os_error().map(|n| n as u32) {
        Some(ERROR_ACCESS_DENIED) => Err(Box::new(HostNameError::SetHostNameDenied)),

        Some(ERROR_INVALID_PARAMETER | ERROR_INVALID_NAME) => {
            Err(Box::new(HostNameError::HostNameTooLong))
        }

        _ => Err(err.into()),
    }
}

pub(crate) fn get_name_info(
    address: *const SOCKADDR,
    address_size: usize,
    flags: c_int,
) -> std::io::Result<OsString> {
    use std::io::{Error, ErrorKind};

    let Ok(address_size) = socklen_t::try_from(address_size) else {
        return Err(Error::from(ErrorKind::InvalidInput));
    };

    let mut buffer: Vec<u16> = vec![0_u16; 1025];

    loop {
        let Ok(buffer_len) = u32::try_from(buffer.len()) else {
            return Err(Error::from_raw_os_error(WSAENAMETOOLONG));
        };

        let r = unsafe {
            GetNameInfoW(
                address,
                address_size,
                buffer.as_mut_ptr(),
                buffer_len,
                ptr::null_mut(),
                0,
                flags,
            )
        };

        match r {
            WSAENOBUFS | WSAENAMETOOLONG => {}

            0 => {
                if let Some(index) = buffer.iter().position(|&ch| ch == 0_u16) {
                    buffer.truncate(index);
                    break Ok(OsString::from_wide(&buffer));
                }
            }

            _ => break Err(Error::from_raw_os_error(unsafe { WSAGetLastError() })),
        }

        buffer.resize_with(buffer.len() + 4096, Default::default);
    }
}

fn get_adapters_addresses() -> std::io::Result<Option<Box<[u8]>>> {
    let mut buffer: Vec<u8> = Vec::with_capacity(32768);

    loop {
        let mut buffer_size = buffer.capacity().min(u32::MAX as usize) as u32;

        let r = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC as u32,
                GAA_FLAG_SKIP_FRIENDLY_NAME | GAA_FLAG_SKIP_DNS_SERVER,
                ptr::null(),
                buffer.spare_capacity_mut().as_mut_ptr().cast(),
                &mut buffer_size,
            )
        };

        match r {
            ERROR_NO_DATA => return Ok(None), // No interface addresses were found.

            ERROR_BUFFER_OVERFLOW => {
                if buffer_size == 0 {
                    return Ok(None); // No interface addresses were found.
                }

                buffer.reserve((buffer_size as usize).saturating_sub(buffer.capacity()));
                // Loop to try again.
            }

            ERROR_SUCCESS => {
                if buffer_size == 0 {
                    return Ok(None); // No interface addresses were found.
                }

                unsafe { buffer.set_len(buffer_size as usize) };
                return Ok(Some(buffer.into_boxed_slice()));
            }

            _ => return Err(std::io::Error::from_raw_os_error(r as i32)),
        };
    }
}

pub(crate) struct InterfaceAddresses {
    list: *const IP_ADAPTER_ADDRESSES_LH,
    buffer: *mut [u8],
}

impl InterfaceAddresses {
    pub(crate) fn new() -> std::io::Result<Self> {
        if let Some(buffer) = get_adapters_addresses()? {
            let list = (*buffer).as_ptr().cast();
            let buffer = Box::into_raw(buffer);
            Ok(Self { list, buffer })
        } else {
            Ok(Self {
                list: ptr::null(),
                buffer: ptr::null_mut::<[u8; 0]>(),
            })
        }
    }

    pub(crate) fn iter(&self) -> InterfaceAddressesIter {
        InterfaceAddressesIter {
            _ia: self,
            ptr: self.list,
        }
    }
}

impl Drop for InterfaceAddresses {
    fn drop(&mut self) {
        if !self.buffer.is_null() {
            drop(unsafe { Box::from_raw(self.buffer) });
        }
    }
}

pub(crate) struct InterfaceAddressesIter<'ia> {
    _ia: &'ia InterfaceAddresses,
    ptr: *const IP_ADAPTER_ADDRESSES_LH,
}

impl<'ia> Iterator for InterfaceAddressesIter<'ia> {
    type Item = &'ia IP_ADAPTER_ADDRESSES_LH;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }

        let element = unsafe { &*self.ptr };
        self.ptr = element.Next;
        Some(element)
    }
}

pub(crate) struct AdapterUnicastAddressIter<'aa> {
    _aa: &'aa IP_ADAPTER_ADDRESSES_LH,
    ptr: *const IP_ADAPTER_UNICAST_ADDRESS_LH,
}

impl<'aa> AdapterUnicastAddressIter<'aa> {
    pub(crate) fn new(adapter_addresses: &'aa IP_ADAPTER_ADDRESSES_LH) -> Self {
        Self {
            _aa: adapter_addresses,
            ptr: adapter_addresses.FirstUnicastAddress,
        }
    }
}

impl<'ia> Iterator for AdapterUnicastAddressIter<'ia> {
    type Item = &'ia IP_ADAPTER_UNICAST_ADDRESS_LH;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }

        let element = unsafe { &*self.ptr };
        self.ptr = element.Next;
        Some(element)
    }
}

#[repr(transparent)]
pub(crate) struct AddressInfo(NonNull<ADDRINFOW>);

impl AddressInfo {
    pub(crate) fn new(
        host_name: &OsStr,
        hint_family: c_int,
        hint_socktype: c_int,
        hint_protocol: c_int,
        hint_flags: c_int,
    ) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};

        let mut host_name: Vec<u16> = host_name.encode_wide().collect();
        host_name.push(0);

        let mut c_hints: ADDRINFOW = unsafe { std::mem::zeroed() };
        c_hints.ai_family = hint_family;
        c_hints.ai_socktype = hint_socktype;
        c_hints.ai_protocol = hint_protocol;
        c_hints.ai_flags = hint_flags;

        let mut ptr: *mut ADDRINFOW = ptr::null_mut();

        if unsafe { GetAddrInfoW(host_name.as_ptr(), ptr::null(), &c_hints, &mut ptr) } == 0 {
            NonNull::new(ptr)
                .map(Self)
                .ok_or_else(|| Error::from(ErrorKind::InvalidData))
        } else {
            Err(Error::from_raw_os_error(unsafe { WSAGetLastError() }))
        }
    }

    pub(crate) fn iter(&self) -> AddressInfoIter {
        AddressInfoIter {
            _ia: self,
            ptr: Some(self.0),
        }
    }
}

impl Drop for AddressInfo {
    fn drop(&mut self) {
        unsafe { FreeAddrInfoW(self.0.as_ptr()) }
    }
}

pub(crate) struct AddressInfoIter<'ia> {
    _ia: &'ia AddressInfo,
    ptr: Option<NonNull<ADDRINFOW>>,
}

impl<'ia> Iterator for AddressInfoIter<'ia> {
    type Item = &'ia ADDRINFOW;

    fn next(&mut self) -> Option<Self::Item> {
        let element = unsafe { self.ptr?.as_ref() };
        self.ptr = NonNull::new(element.ai_next);
        Some(element)
    }
}

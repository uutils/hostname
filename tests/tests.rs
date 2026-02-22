#[cfg(feature = "hostname")]
#[path = "by_util/hostname.rs"]
mod test_hostname;

#[cfg(feature = "domainname")]
#[path = "by_util/domainname.rs"]
mod test_domainname;

#[cfg(feature = "dnsdomainname")]
#[path = "by_util/domainname.rs"]
mod test_dnsdomainname;

#[cfg(feature = "nisdomainname")]
#[path = "by_util/domainname.rs"]
mod test_nisdomainname;

#[cfg(feature = "ypdomainname")]
#[path = "by_util/domainname.rs"]
mod test_ypdomainname;
use nix::ifaddrs::{getifaddrs};
use nix::sys::socket::AddressFamily;
use tracing::{info};

pub fn get_one_host_ip_address()-> String {
    let addrs = getifaddrs().unwrap();
    let mut addresses = vec![];
    for ifaddr in addrs {
        match ifaddr.address {
            Some(address) => {
                if address.family() == AddressFamily::Inet {
                    addresses.push(address.to_str());
                }
            },
            None => {
                info!("cannot retrieve ip address,  interface {} with unsupported address family",
                             ifaddr.interface_name);
            }
        }
    };

    if addresses.len() > 1 && addresses[0].starts_with("127.0.0.1") {
        return String::from(addresses[1].split(":").next().unwrap())
    }
    String::from(addresses[0].split(":").next().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_ipv4(address: &str)->bool{
        let parts = address.split(".").collect::<Vec<&str>>();
        parts.len() == 4
    }

    #[test]
    fn should_return_ipv4_address() {
        let address = get_one_host_ip_address();

        assert!(is_ipv4(address.as_str()))
    }
}

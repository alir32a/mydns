use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use crate::handler::HandlerTarget;

pub struct RootServer(&'static str, Ipv4Addr, Ipv6Addr);

impl RootServer {
    pub fn to_socket_addrs(&self) -> (HandlerTarget, HandlerTarget) {
        (
            HandlerTarget::from_addr(SocketAddr::new(IpAddr::V4(self.1), 53)),
            HandlerTarget::from_addr(SocketAddr::new(IpAddr::V6(self.2), 53)),
        )
    }
}

pub static ROOT_SERVERS: [RootServer; 13] = [
    RootServer(
        "a.root-servers.net",
        Ipv4Addr::new(198,41,0,4),
        Ipv6Addr::new(0x2001,0x0503,0xba3e,0,0,0,0x0002,0x0030)
    ),
    RootServer(
        "b.root-servers.net",
        Ipv4Addr::new(170,247,170,2),
        Ipv6Addr::new(0x2801,0x01b8,0x0010,0,0,0,0,0x000b)
    ),
    RootServer(
        "c.root-servers.net",
        Ipv4Addr::new(192,33,4,12),
        Ipv6Addr::new(0x2001, 0x0500, 0x0002, 0, 0, 0, 0, 0x000c)
    ),
    RootServer(
        "d.root-servers.net",
        Ipv4Addr::new(199,7,91,13),
        Ipv6Addr::new(0x2001, 0x0500, 0x002d, 0, 0, 0, 0, 0x000d)
    ),
    RootServer(
        "e.root-servers.net",
        Ipv4Addr::new(192,203,230,10),
        Ipv6Addr::new(0x2001, 0x0500, 0x00a8, 0, 0, 0, 0, 0x000e)
    ),
    RootServer(
        "f.root-servers.net",
        Ipv4Addr::new(192,5,5,241),
        Ipv6Addr::new(0x2001, 0x0500, 0x002f, 0, 0, 0, 0, 0x000f)
    ),
    RootServer(
        "g.root-servers.net",
        Ipv4Addr::new(192,112,36,4),
        Ipv6Addr::new(0x2001, 0x0500, 0x0012, 0, 0, 0, 0, 0x0d0d)
    ),
    RootServer(
        "h.root-servers.net",
        Ipv4Addr::new(198,97,190,53),
        Ipv6Addr::new(0x2001, 0x0500, 0x0001, 0, 0, 0, 0, 0x0053)
    ),
    RootServer(
        "i.root-servers.net",
        Ipv4Addr::new(192,36,148,17),
        Ipv6Addr::new(0x2001, 0x07fe, 0, 0, 0, 0, 0, 0x0053)
    ),
    RootServer(
        "j.root-servers.net",
        Ipv4Addr::new(192,58,128,30),
        Ipv6Addr::new(0x2001, 0x0503, 0x0c27, 0, 0, 0, 0x0002, 0x0030)
    ),
    RootServer(
        "k.root-servers.net",
        Ipv4Addr::new(193,0,14,129),
        Ipv6Addr::new(0x2001, 0x07fd, 0, 0, 0, 0, 0, 0x0001)
    ),
    RootServer(
        "l.root-servers.net",
        Ipv4Addr::new(193,0,14,129),
        Ipv6Addr::new(0x2001, 0x0500, 0x009f, 0, 0, 0, 0, 0x0042)
    ),
    RootServer(
        "m.root-servers.net",
        Ipv4Addr::new(202,12,27,33),
        Ipv6Addr::new(0x2001, 0x0dc3, 0, 0, 0, 0, 0, 0x0035)
    ),
];

pub fn get_root_servers_socket_addrs(use_v6: bool) -> Vec<HandlerTarget> {
    let mut res = Vec::new();
    for server in &ROOT_SERVERS {
        let (v4, v6) = server.to_socket_addrs();
        res.push(v4);

        if use_v6 {
            res.push(v6)
        }
    }

    res
}
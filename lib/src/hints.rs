use std::net::{Ipv4Addr, Ipv6Addr};

use rand::prelude::*;


pub fn random_root_server() -> &'static ServerInfo {
    ROOT_SERVERS
        .choose(&mut rand::rng())
        .expect("list of root servers should be composed of 13 servers")
}

pub struct ServerInfo {
    pub name: &'static str,
    pub operator: &'static str,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
}

pub const ROOT_SERVERS: &[ServerInfo] = &[
    ServerInfo {
        name: "a.root-servers.net",
        operator: "Verisign, Inc.",
        ipv4: Ipv4Addr::new(198, 41, 0, 4),
        ipv6: Ipv6Addr::new(0x2001, 0x503, 0xba3e, 0, 0, 0, 0x2, 0x30),
    },
    ServerInfo {
        name: "b.root-servers.net",
        operator: "University of Southern California, Information Sciences Institute",
        ipv4: Ipv4Addr::new(170, 247, 170, 2),
        ipv6: Ipv6Addr::new(0x2801, 0x1b8, 0x10, 0, 0, 0, 0, 0xc),
    },
    ServerInfo {
        name: "c.root-servers.net",
        operator: "Cogent Communications",
        ipv4: Ipv4Addr::new(192, 33, 4, 12),
        ipv6: Ipv6Addr::new(0x2001, 0x500, 0x2, 0, 0, 0, 0, 0xc),
    },
    ServerInfo {
        name: "d.root-servers.net",
        operator: "University of Maryland",
        ipv4: Ipv4Addr::new(199, 7, 91, 13),
        ipv6: Ipv6Addr::new(0x2001, 0x500, 0x2d, 0, 0, 0, 0, 0xd),
    },
    ServerInfo {
        name: "e.root-servers.net",
        operator: "NASA (Ames Research Center)",
        ipv4: Ipv4Addr::new(192, 203, 230, 10),
        ipv6: Ipv6Addr::new(0x2001, 0x500, 0xa8, 0, 0, 0, 0, 0xe),
    },
    ServerInfo {
        name: "f.root-servers.net",
        operator: "Internet Systems Consortium, Inc.",
        ipv4: Ipv4Addr::new(192, 5, 5, 241),
        ipv6: Ipv6Addr::new(0x2001, 0x500, 0x2f, 0, 0, 0, 0, 0xf),
    },
    ServerInfo {
        name: "g.root-servers.net",
        operator: "US Department of Defense (NIC)",
        ipv4: Ipv4Addr::new(192, 112, 36, 4),
        ipv6: Ipv6Addr::new(0x2001, 0x500, 0x12, 0, 0, 0, 0, 0xd0d),
    },
    ServerInfo {
        name: "h.root-servers.net",
        operator: "US Army (Research Lab)",
        ipv4: Ipv4Addr::new(198, 97, 190, 53),
        ipv6: Ipv6Addr::new(0x2001, 0x500, 0x1, 0, 0, 0, 0, 0x53),
    },
    ServerInfo {
        name: "i.root-servers.net",
        operator: "Netnod",
        ipv4: Ipv4Addr::new(192, 36, 148, 17),
        ipv6: Ipv6Addr::new(0x2001, 0x7fe, 0, 0, 0, 0, 0, 0x53),
    },
    ServerInfo {
        name: "j.root-servers.net",
        operator: "Verisign, Inc.",
        ipv4: Ipv4Addr::new(192, 58, 128, 30),
        ipv6: Ipv6Addr::new(0x2001, 0x503, 0xc27, 0, 0, 0, 0x2, 0x30),
    },
    ServerInfo {
        name: "k.root-servers.net",
        operator: "RIPE NCC",
        ipv4: Ipv4Addr::new(193, 0, 14, 129),
        ipv6: Ipv6Addr::new(0x2001, 0x7fd, 0, 0, 0, 0, 0, 0x1),
    },
    ServerInfo {
        name: "l.root-servers.net",
        operator: "ICANN",
        ipv4: Ipv4Addr::new(199, 7, 83, 42),
        ipv6: Ipv6Addr::new(0x2001, 0x500, 0x9f, 0, 0, 0, 0, 0x42),
    },
    ServerInfo {
        name: "m.root-servers.net",
        operator: "WIDE Project",
        ipv4: Ipv4Addr::new(202, 12, 27, 33),
        ipv6: Ipv6Addr::new(0x2001, 0xdc3, 0, 0, 0, 0, 0, 0x35),
    },
];

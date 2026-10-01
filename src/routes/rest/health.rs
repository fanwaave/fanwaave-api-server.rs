#![forbid(unsafe_code)]

use fanwaave_interfaces::{Health, PROTOCOL_VERSION};

pub fn body() -> Health {
    Health {
        ok: true,
        service: "fanwaave-api-server".to_owned(),
        protocol: PROTOCOL_VERSION.to_owned(),
    }
}

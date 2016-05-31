// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use cert::CertType;
use czmq::ZFrame;
use error::{Error, Result};

pub struct RequestMeta {
    // id: u32,
    pub name: String,
    pub cert_type: CertType,
}

impl RequestMeta {
    pub fn new(frame: &ZFrame) -> Result<RequestMeta> {
        Ok(RequestMeta {
            // id: if let Ok(Some(Ok(id))) = frame.meta("id") {
            //         match id.parse::<u32>() {
            //             Ok(int) => int,
            //             Err(_) => return Err(Error::InvalidCert),
            //         }
            //     } else {
            //         return Err(Error::InvalidCert);
            //     },
            name: if let Some(Ok(name)) = frame.meta("name") {
                    name
                } else {
                    return Err(Error::InvalidCert);
                },
            cert_type: if let Some(Ok(ctype)) = frame.meta("type") {
                    try!(CertType::from_str(&ctype))
                } else {
                    return Err(Error::InvalidCert);
                },
        })
    }
}

#[cfg(test)]
mod tests {
    use czmq::{zsys_init, ZCert, ZFrame, ZMsg, ZSock, ZSockType};
    use super::*;

    #[test]
    fn test_new() {
        zsys_init();

        let zap = ZSock::new_rep("inproc://zeromq.zap.01").unwrap();

        let server = ZSock::new(ZSockType::REP);
        server.set_zap_domain("test");
        server.set_curve_server(true);
        let server_cert = ZCert::new().unwrap();
        server_cert.apply(&server);
        let port = server.bind("tcp://127.0.0.1:*[60000-]").unwrap();

        let client = ZSock::new(ZSockType::REQ);
        client.set_curve_serverkey(server_cert.public_txt());
        let client_cert = ZCert::new().unwrap();
        client_cert.set_meta("name", "ben.dover");
        client_cert.set_meta("type", "user");
        client_cert.apply(&client);
        client.connect(&format!("tcp://127.0.0.1:{}", port)).unwrap();

        // Discard ZAP request
        zap.recv_str().unwrap().unwrap();

        let msg = ZMsg::new();
        msg.addstr("1.0").unwrap();
        msg.addstr("1").unwrap();
        msg.addstr("200").unwrap();
        msg.addstr("OK").unwrap();
        msg.addstr("").unwrap(); // User ID
        msg.addbytes(&client_cert.encode_meta()).unwrap();
        msg.send(&zap).unwrap();

        client.send_str("test").unwrap();
        let frame = ZFrame::recv(&server).unwrap();
        assert!(RequestMeta::new(&frame).is_ok());
    }
}

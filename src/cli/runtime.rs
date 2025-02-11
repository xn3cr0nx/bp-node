// Bitcoin protocol (BP) daemon node
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::convert::TryFrom;

use super::*;
use crate::error::BootstrapError;
use crate::msgbus::{self, Multipart};

pub struct Runtime {
    config: Config,
    context: zmq::Context,
    api_socket: zmq::Socket,
    sub_socket: zmq::Socket,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError> {
        let context = zmq::Context::new();

        debug!(
            "Opening API socket to wired on {} ...",
            config.msgbus_peer_api_addr
        );
        let api_socket = context
            .socket(zmq::REQ)
            .map_err(|e| BootstrapError::PublishingError(e))?;
        api_socket
            .bind(&config.msgbus_peer_api_addr)
            .map_err(|e| BootstrapError::PublishingError(e))?;

        debug!(
            "Opening push notification socket to wired on {} ...",
            config.msgbus_peer_sub_addr
        );
        let sub_socket = context
            .socket(zmq::SUB)
            .map_err(|e| BootstrapError::SubscriptionError(e))?;
        sub_socket
            .connect(&config.msgbus_peer_sub_addr)
            .map_err(|e| BootstrapError::SubscriptionError(e))?;
        sub_socket
            .set_subscribe("".as_bytes())
            .map_err(|e| BootstrapError::SubscriptionError(e))?;

        debug!("Console is launched");
        Ok(Self {
            config,
            context,
            api_socket,
            sub_socket,
        })
    }

    pub fn command_query(&self, query: String) -> Result<(), msgbus::Error> {
        info!("Performing QUERY command {} ...", query);
        let multipart: msgbus::Multipart = msgbus::Command::Query(msgbus::Query { query }).into();
        self.api_socket.send_multipart(multipart, 0)?;
        trace!("Request sent, awaiting response ...");
        let rep: Multipart = self
            .api_socket
            .recv_multipart(0)?
            .iter()
            .map(|vec| zmq::Message::from(vec))
            .collect();
        println!("{}", msgbus::Command::try_from(rep)?);
        Ok(())
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        trace!("Shutting down sockets");
        self.api_socket
            .disconnect(&self.config.msgbus_peer_api_addr)
            .unwrap_or_else(|err| error!("Error disconnecting message bus API socket: {}", err));
        self.sub_socket
            .disconnect(&self.config.msgbus_peer_sub_addr)
            .unwrap_or_else(|err| error!("Error disconnecting message bus push socket: {}", err));
    }
}

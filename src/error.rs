// Lightning network protocol (LNP) daemon node
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

use std::io;

#[derive(Debug, Display)]
#[display_from(Debug)]
pub enum BootstrapError {
    TorNotYetSupported,
    IoError(io::Error),
    ArgParseError(String),
    SubscriptionError(zmq::Error),
    PublishingError(zmq::Error),
    MonitorSocketError(Box<dyn std::error::Error>),
}

impl std::error::Error for BootstrapError {}

impl From<BootstrapError> for String {
    fn from(err: BootstrapError) -> Self {
        format!("{}", err)
    }
}

impl From<&str> for BootstrapError {
    fn from(err: &str) -> Self {
        BootstrapError::ArgParseError(err.to_string())
    }
}

impl From<String> for BootstrapError {
    fn from(err: String) -> Self {
        BootstrapError::ArgParseError(err)
    }
}

impl From<io::Error> for BootstrapError {
    fn from(err: io::Error) -> Self {
        BootstrapError::IoError(err)
    }
}

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

use bitcoin::{consensus, secp256k1};

#[derive(Debug, Display)]
#[display_from(Debug)]
pub enum Error {
    MessageBusError(zmq::Error),
    
    // Request-specific errors
    MalformedRequest,
    MalformedCommand,
    UnknownCommand,

    // Reply-specific errors
    MalformedReply,
    MalformedStatus,
    UnknownStatus,

    // General API errors that may happen with both requests and replies
    MalformedArgument,
    WrongNumberOfArguments,
}

impl std::error::Error for Error {}

impl From<Error> for String {
    fn from(err: Error) -> Self {
        format!("{}", err)
    }
}

impl From<zmq::Error> for Error {
    fn from(err: zmq::Error) -> Self {
        Error::MessageBusError(err)
    }
}

impl From<consensus::encode::Error> for Error {
    fn from(_: consensus::encode::Error) -> Self {
        Error::MalformedArgument
    }
}

impl From<secp256k1::Error> for Error {
    fn from(_: secp256k1::Error) -> Self {
        Error::MalformedArgument
    }
}

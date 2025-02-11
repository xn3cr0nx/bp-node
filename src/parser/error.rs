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


use diesel::result::Error as DieselError;
use bitcoin::hashes;


#[derive(PartialEq, Debug, Display)]
#[display_from(Debug)]
pub enum Error {
    IndexIntegrityError,
    IndexError(DieselError),
    CorruptedShortId,
    BlockValidationIncosistency,
}

impl std::error::Error for Error {}

impl From<DieselError> for Error {
    fn from(err: DieselError) -> Self {
        Error::IndexError(err)
    }
}

impl From<hashes::Error> for Error {
    fn from(_: hashes::Error) -> Self {
        Error::IndexIntegrityError
    }
}

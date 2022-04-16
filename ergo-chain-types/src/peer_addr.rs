//! Peer address types
use std::{
    convert::TryInto,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
};

use derive_more::FromStr;
use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};
use sigma_ser::{ScorexSerializable, ScorexSerializationError};
use url::Url;

/// Peer address
#[derive(
    PartialEq, Eq, Debug, Copy, Clone, From, Into, Hash, Display, FromStr, Deserialize, Serialize,
)]
pub struct PeerAddr(pub SocketAddr);

impl PeerAddr {
    /// Size in bytes of the ip address associated with this peer address
    pub fn ip_size(&self) -> usize {
        match self.0.ip() {
            IpAddr::V4(ip) => ip.octets().len(),
            IpAddr::V6(ip) => ip.octets().len(),
        }
    }

    /// Build an http://address:port/ URL
    pub fn as_http_url(&self) -> Url {
        let s: String =
            "http://".to_string() + &self.0.ip().to_string() + ":" + &self.0.port().to_string();
        #[allow(clippy::unwrap_used)]
        Url::from_str(&s).unwrap()
    }
}

impl ScorexSerializable for PeerAddr {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> sigma_ser::ScorexSerializeResult {
        let ip = match self.0.ip() {
            IpAddr::V4(ip) => ip,
            _ => return Err(ScorexSerializationError::NotSupported("ipv6 not supported")),
        };

        w.write_all(&ip.octets())?;
        w.put_u32(self.0.port() as u32)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        let mut fa = [0u8; 4];
        r.read_exact(&mut fa)?;

        let ip = Ipv4Addr::from(fa);
        let port: u16 = r.get_u32()?.try_into()?;

        Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)).into())
    }
}

/// Arbitrary
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    use proptest::prelude::*;
    use proptest::prelude::{Arbitrary, BoxedStrategy};

    impl Arbitrary for PeerAddr {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (any::<[u8; 4]>(), any::<u16>())
                .prop_map(|(octets, port)| {
                    SocketAddr::new(Ipv4Addr::from(octets).into(), port).into()
                })
                .boxed()
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::scorex_serialize_roundtrip;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn ser_roundtrip(v in any::<PeerAddr>()) {
            assert_eq![scorex_serialize_roundtrip(&v), v]
        }
    }
}

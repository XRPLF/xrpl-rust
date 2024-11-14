use alloc::borrow::Cow;

use crate::{
    core::addresscodec::encode_classic_address,
    models::{transactions::nftoken_mint::NFTokenMintFlag, FlagCollection},
    utils::exceptions::XRPLNFTIdException,
};

use super::exceptions::XRPLUtilsResult;

pub struct NFTokenId<'a> {
    pub nftoken_id: Cow<'a, str>,
    pub flags: FlagCollection<NFTokenMintFlag>,
    pub transfer_fee: u32,
    pub issuer: Cow<'a, str>,
    pub taxon: u64,
    pub sequence: u32,
}

/// Unscrambles or rescrambles a taxon in an NFTokenID.
///
/// An issuer may issue several NFTs with the same taxon; to ensure that NFTs
/// are spread across multiple pages we lightly mix the taxon up by using the
/// sequence (which is not under the issuer's direct control) as the seed for
/// a simple linear congruential generator.
///
/// From the Hull-Dobell theorem we know that f(x)=(m*x+c) mod n will yield a
/// permutation of [0, n) when n is a power of 2 if m is congruent to 1 mod 4
/// and c is odd. By doing a bitwise XOR with this permutation we can
/// scramble/unscramble the taxon.
///
/// The XLS-20d proposal fixes m = 384160001 and c = 2459.
/// We then take the modulus of 2^32 which is 4294967296.
pub fn unscramble_taxon(taxon: u64, token_seq: u64) -> u64 {
    return (taxon ^ (384160001 * token_seq + 2459)) % 4294967296;
}

/// Parse an NFTokenID into the information it is encoding.
///
/// Example decoding:
///
/// 000B 0539 C35B55AA096BA6D87A6E6C965A6534150DC56E5E 12C5D09E 0000000C
/// +--- +--- +--------------------------------------- +------- +-------
/// |    |    |                                        |        |
/// |    |    |                                        |        `---> Sequence: 12
/// |    |    |                                        |
/// |    |    |                                        `---> Scrambled Taxon: 314,953,886
/// |    |    |                                              Unscrambled Taxon: 1337
/// |    |    |
/// |    |    `---> Issuer: rJoxBSzpXhPtAuqFmqxQtGKjA13jUJWthE
/// |    |
/// |    `---> TransferFee: 1337.0 bps or 13.37%
/// |
/// `---> Flags: 11 -> lsfBurnable, lsfOnlyXRP and lsfTransferable
pub fn parse_nftoken_id(nft_id: Cow<str>) -> XRPLUtilsResult<NFTokenId<'_>> {
    const EXPECTED_LEN: usize = 64;

    if nft_id.len() != EXPECTED_LEN {
        return Err(XRPLNFTIdException::InvalidNFTIdLength {
            expected: EXPECTED_LEN,
            found: nft_id.len(),
        }
        .into());
    }
    let scrambled_taxon = u64::from_str_radix(&nft_id[48..56], 16)?;
    let sequence = u32::from_str_radix(&nft_id[56..64], 16)?;
    let flags = u32::from_str_radix(&nft_id[0..4], 16)?;
    let transfer_fee = u32::from_str_radix(&nft_id[4..8], 16)?;
    let issuer_bytes = hex::decode(&nft_id[8..48])?;
    let issuer = encode_classic_address(issuer_bytes.as_ref())?;

    Ok(NFTokenId {
        nftoken_id: nft_id,
        flags: flags.try_into()?,
        transfer_fee,
        issuer: issuer.into(),
        taxon: unscramble_taxon(scrambled_taxon, sequence as u64),
        sequence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nftoken_id() {
        let nft_id = "000B0539C35B55AA096BA6D87A6E6C965A6534150DC56E5E12C5D09E0000000C";
        let nftoken_id = parse_nftoken_id(Cow::Borrowed(nft_id)).unwrap();
        assert_eq!(nftoken_id.flags.len(), 3);
        assert_eq!(nftoken_id.transfer_fee, 1337);
        assert_eq!(nftoken_id.issuer, "rJoxBSzpXhPtAuqFmqxQtGKjA13jUJWthE");
        assert_eq!(nftoken_id.taxon, 1337);
        assert_eq!(nftoken_id.sequence, 12);
    }
}

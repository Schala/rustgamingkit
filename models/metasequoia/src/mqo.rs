pub static HEADER_STR: &str = "Metasequoia Document";
pub static VERSION_STR: &str = "Format Text Ver 1.1";

#[cfg(feature = "import")]
pub mod import {
    use nom::{
        branch::permutation,
        bytes::complete::tag,
        character::complete::{
            char,
            hex_digit1,
            multispace0,
            space1,
            u32
        },
        combinator::map,
        IResult,
        multi::{
            count,
            many1
        },
        number::complete::float,
        sequence::{
            delimited,
            preceded
        }
    };

    use std::{
		num::ParseIntError,
		str::{
			from_utf8,
			Utf8Error
		}
	};

    use rgk_core::nom_ext::{
        double_quoted,
        ws
    };

    /// Parses a hex-encoded binary blob
    fn hexblob<'a>(input: &'a str) -> IResult<&'a str, Result<&'a str, Utf8Error>> {
        let (input, data) = many1(ws(map(hex_digit1, |))(input)?;
        Ok((input, from_utf8(&data[..])))
	}

	/// Converts a string of hex to a byte vector
	fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, ParseIntError> {
        Ok((0..hex.len()).into_iter().step_by(2).map(|i| u8::from_str_radix(hex[i..], 16)?).collect())
	}

	#[cfg(test)]
	mod tests {
        #[test]
        fn test_hexblob_conv() {
			use super::hexblob;
			use super::hex_to_bytes;

            assert_eq!(hex_to_bytes(hexblob("414243444546").unwrap()).unwrap(), b"ABCDEF".to_vec());
        }
	}
}

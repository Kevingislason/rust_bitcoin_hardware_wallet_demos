//Adapted from Jimmy Song's Programming Bitcoin library:
//https://github.com/jimmysong/programmingbitcoin/

use num::bigint::BigInt;
use num::bigint::Sign::Plus;
use num::ToPrimitive;
use num_traits::identities::{One, Zero};
use ripemd160::Ripemd160;
use sha2::{Digest, Sha256};
use crate::cursor::Cursor;
use crate::genio::Read;
use crate::serialization::Serialization;

pub const SIGHASH_ALL: u8 = 1;
pub const SIGHASH_NONE: u8 = 2;
pub const SIGHASH_SINGLE: u8 = 3;

pub fn hash_160(bytes: Vec<u8>) -> Vec<u8> {
  //Hash our bytes with sha256, then ripemd160
  let mut sha256 = Sha256::new();
  sha256.input(bytes);
  let sha256_result = sha256.result();
  let mut ripemd160 = Ripemd160::new();
  ripemd160.input(sha256_result);
  ripemd160.result().to_vec()
}

pub fn hash_256(bytes: Vec<u8>) -> Vec<u8> {
  let mut sha256 = Sha256::new();
  sha256.input(bytes);
  let sha256_result = sha256.result();
  let mut sha256 = Sha256::new();
  sha256.input(sha256_result);
  sha256.result().to_vec()
}

pub fn encode_base58(bytes: Vec<u8>) -> String {
  let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
  let mut count = 0;
  for c in bytes.clone() {
    if c == 0 {
      count += 1;
    } else {
      break;
    }
  }
  let mut num = BigInt::from_bytes_be(Plus, &bytes);
  let mut prefix = String::from("");
  for _ in 0..count {
    prefix += "1";
  }
  let mut result = String::from("");
  while &num > &BigInt::zero() {
    let modulo: usize = (&num % 58u8).to_usize().unwrap();
    num = &num / 58;
    let character = String::from(&base58_alphabet[modulo..modulo + 1]);
    result = character + &result;
  }
  return prefix + &result;
}

pub fn encode_base58_checksum(bytes: Vec<u8>) -> String {
  let mut unencoded = bytes.clone();
  let mut last_4_bytes_of_hash = hash_256(bytes)[0..4].to_vec();
  unencoded.append(&mut last_4_bytes_of_hash);
  encode_base58(unencoded)
}

//todo: refactor, I think this code is bad
//todo: write tests for this
pub fn decode_base58(s: String) -> Vec<u8> {
  let base58_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
  let mut num = BigInt::zero();
  for c in s.chars() {
    num *= 58;
    num = num + &base58_alphabet.find(c).unwrap();
  }
  let mut combined = num.to_bytes_be().1;
  //pad to 25 bytes
  if combined.len() < 25 {
    let padding_len = 25 - combined.len();
    let mut padded_combined: Vec<u8> = vec![0; padding_len];
    padded_combined.append(&mut combined);
    combined = padded_combined;
  }

  let checksum = combined[21..25].to_vec();
  let first_21_chars_of_combined = combined[0..21].to_vec();
  let hash = hash_256(first_21_chars_of_combined);
  if hash[0..4].to_vec() != checksum {
    panic!("bad address: {:?} {:?}", checksum, hash[0..4].to_vec());
  }
  return combined[1..21].to_vec();
}

pub fn read_varint(cursor: &mut Cursor<Vec<u8>>) -> u64 {
  let mut buffer = [0; 1];
  cursor.read(&mut buffer).unwrap();
  match buffer[0] {
    0xfd => cursor.read_u16_little_endian().unwrap() as u64,
    0xfe => cursor.read_u32_little_endian().unwrap() as u64,
    0xff => cursor.read_u64_little_endian().unwrap(),
    _ => buffer[0].into(),
  }
}

pub fn encode_varint(i: u64) -> Vec<u8> {
  let mut serialization = Serialization::new();
  if i < 0xfd {
    vec![i as u8]
  } else if i < 0x10000 {
    serialization.write_u16_little_endian(i as u16).unwrap();
    //just concatenating here lol rust
    vec![0xfd].into_iter().chain(serialization.contents.into_iter()).collect()
  } else if i < 0x100000000 {
    serialization.write_u32_little_endian(i as u32).unwrap();
    vec![0xfe].into_iter().chain(serialization.contents.into_iter()).collect()
  } else {
    serialization.write_u64_little_endian(i as u64).unwrap();
    vec![0xff].into_iter().chain(serialization.contents.into_iter()).collect()
  }
}

#[test]
pub fn test_enocde_base58() {
  println!("{}", 100u8 / 58u8);
  let bytes: Vec<u8> = vec![1, 2, 200, 255, 122];
  assert_eq!(encode_base58(bytes), "7cfKTo")
}

#[test]
pub fn test_decode_base58() {
  let address = String::from("mnrVtF8DWjMu839VW3rBfgYaAfKk8983Xf");
  let h160 = hex::encode(decode_base58(address));
  let want = "507b27411ccf7f16f10297de6cef3f291623eddf";
  assert_eq!(h160, want);
}


#[test]
pub fn test_varint() {
  assert_eq!(encode_varint(256u64), vec![0xfd, 0, 1]);

  assert_eq!(encode_varint(0x100ffu64), vec![0xfe, 0xff, 0, 1, 0]);

  assert_eq!(
    encode_varint(0xffffffffffffffffu64),
    vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
  );

  assert_eq!(
    encode_varint(18005558675309),
    vec![0xff, 0x6d, 0xc7, 0xed, 0x3e, 0x60, 0x10, 0, 0]
  );

  assert_eq!(read_varint(&mut Cursor::new(encode_varint(100))), 100);
  assert_eq!(read_varint(&mut Cursor::new(encode_varint(1000))), 1000);
  assert_eq!(read_varint(&mut Cursor::new(encode_varint(100000))), 100000);
  assert_eq!(
    read_varint(&mut Cursor::new(encode_varint(1000000000000000))),
    1000000000000000
  );

  let bigger_than_a_varint = vec![
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x33, 0x33, 0x33, 0x33,
  ];
  let biggest_varint = vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

  assert_eq!(
    read_varint(&mut Cursor::new(biggest_varint)),
    read_varint(&mut Cursor::new(bigger_than_a_varint))
  );
}

//! VMess key derivation (v2ray `proxy/vmess/aead` + `common/protocol/id.go`).
use md5::Md5;
use md5::digest::Digest as _;
use sha2::Sha256;

pub const VMESS_SALT: &str = "c48619fe-8f02-49e0-b9e9-edf763e17e21";
const KDF_ROOT: &str = "VMess AEAD KDF";

/// cmdKey = md5(uuid_bytes ‖ VMESS_SALT) — Go `protocol.NewID`.
pub fn cmd_key(uuid_bytes: &[u8; 16]) -> [u8; 16] {
    let mut m = Md5::new();
    m.update(uuid_bytes);
    m.update(VMESS_SALT.as_bytes());
    m.finalize().into()
}

/// Nested HMAC-SHA256: `HMAC(p[n], … HMAC(p[1], HMAC(KDF_ROOT, key)))`.
/// Go `aead.KDF` — message chains root → key at the innermost position.
pub fn kdf(key: &[u8], path: &[&str]) -> [u8; 32] {
    let mut msg: [u8; 32] = hmac_sha256(KDF_ROOT.as_bytes(), key);
    for v in path {
        msg = hmac_sha256(v.as_bytes(), &msg);
    }
    msg
}

pub fn kdf16(key: &[u8], path: &[&str]) -> [u8; 16] {
    let full = kdf(key, path);
    let mut out = [0u8; 16];
    out.copy_from_slice(&full[..16]);
    out
}

fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    use hmac::{Hmac, Mac};
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("hmac key any size");
    mac.update(msg);
    mac.finalize().into_bytes().into()
}

/// FNV-1a 32-bit (Go `hash/fnv`).
pub fn fnv1a32(data: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in data {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// CRC-32 IEEE (Go `hash/crc32`).
pub fn crc32_ieee(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// Request AuthID: AES-128-ECB(key = KDF16(cmdKey, "AES Auth ID Encryption"),
/// block = [ts_be][rand4][crc32(ts_be ‖ rand4)]). Go `aead.CreateAuthID`.
pub fn auth_id(cmd_key: &[u8; 16], ts_secs: i64, rand4: &[u8; 4]) -> [u8; 16] {
    use aes::cipher::{BlockEncrypt, KeyInit};
    let key = kdf16(cmd_key, &["AES Auth ID Encryption"]);
    let mut block = [0u8; 16];
    block[..8].copy_from_slice(&ts_secs.to_be_bytes());
    block[8..12].copy_from_slice(rand4);
    let crc = crc32_ieee(&block[..12]);
    block[12..].copy_from_slice(&crc.to_be_bytes());
    let cipher = aes::Aes128::new_from_slice(&key).expect("16-byte key");
    let mut out = [0u8; 16];
    cipher.encrypt_block_b2b((&block).into(), (&mut out).into());
    out
}

/// sha256(data)[..16] — responseBodyKey/IV derivation (Go `crypto/sha256`).
pub fn sha256_first16(data: &[u8]) -> [u8; 16] {
    let d = Sha256::digest(data);
    let mut out = [0u8; 16];
    out.copy_from_slice(&d[..16]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO_UUID: [u8; 16] = [0; 16];

    #[test]
    fn cmd_key_matches_go_id_new() {
        // Go: NewID(uuid.Zero) cmdKey (computed via md5(uuid ‖ const))
        assert_eq!(
            hex(&cmd_key(&ZERO_UUID)),
            "5e20f3239545e3f48e0ff445aa7c4c3b"
        );
    }

    #[test]
    fn kdf_matches_go_nested_hmac() {
        let ck = cmd_key(&ZERO_UUID);
        // Go aead.KDF16(ck, "AES Auth ID Encryption")
        assert_eq!(hex(&kdf16(&ck, &["AES Auth ID Encryption"])), "e4e63970ccc1e39dd4a315d51b66abf7");
        // Go aead.KDF16(ck, "AEAD Resp Header Len Key")
        assert_eq!(hex(&kdf16(&ck, &["AEAD Resp Header Len Key"])), "2e2b7503835a5934f683a5f84910e211");
    }

    #[test]
    fn fnv1a32_matches_go() {
        // sample request-body prefix used for the golden: version|iv|key|resp|opt|sec|0|cmd|addr
        let mut body = vec![1u8];
        body.extend_from_slice(&[0x11; 16]);
        body.extend_from_slice(&[0x22; 16]);
        body.extend_from_slice(&[0x33, 0x00, 0x00, 0x00, 0x01]);
        body.extend_from_slice(&[0x01, 127, 0, 0, 1]);
        body.extend_from_slice(&0u16.to_be_bytes());
        assert_eq!(fnv1a32(&body), 0x01e79ab9);
    }

    #[test]
    fn auth_id_encrypts_ecb_golden() {
        let ck = cmd_key(&ZERO_UUID);
        // ts=0x6000000000000000, rand=0xaabbccdd -> crc32 0x277774dc; key = KDF16(ck, salt)
        // golden ciphertext (openssl aes-128-ecb): 5659b5bba7891c58aad644346470c210
        let id = auth_id(&ck, 0x6000_0000_0000_0000, &[0xaa, 0xbb, 0xcc, 0xdd]);
        assert_eq!(hex(&id), "5659b5bba7891c58aad644346470c210");
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
}

use core::str::from_utf8;
use alloc::sync::Arc;
use alloc::{boxed::Box, string::{String, ToString}, vec::Vec,vec};
use polyval::{Polyval, Key as PolyvalKey};
use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use dyn_clone::{clone_trait_object, DynClone};
use universal_hash::{generic_array, UniversalHash}; 
use crate::{net::socket::SockResult, syscall::{SysError, SysResult}};
use salsa20::{cipher::{KeyIvInit, StreamCipher}, Salsa20};
use crate::syscall::fs::IoVec;
/// api for both user space an kernel space
#[repr(C)]
#[derive(Debug,Clone,Copy)]
pub struct SockAddrAlg {
    /// address family: af_alg
    pub salg_family: u16,
    /// algorithm name: should be "skcipher", "hash" and so on
    pub salg_type: [u8; 14],
    /// feat: usually 0
    pub salg_feat: u32,
    /// mask: usually 0
    pub salg_mask: u32,
    /// name: the name of the algorithm
    pub salg_name: [u8; 64],
}

impl SockAddrAlg {
    pub fn get_name(&self) -> Result<&str, SysError> {
        let end = self.salg_name.iter().position(|&bytes| bytes == 0).unwrap_or(14);
        match from_utf8(&self.salg_name[..end]) {
            Ok(string) => return Ok(string),
            Err(_) => return Err(SysError::EINVAL),
        }
    }
    pub fn check_alg(&self) -> SockResult<isize> {
        /// check type
        let type_name_end = self.salg_type
            .iter().position(|&bytes| bytes == 0)
            .unwrap_or(self.salg_type.len());
        let raw_type = &self.salg_type[..type_name_end];
        let alg_type = match from_utf8(raw_type) {
            Ok(string) => string,
            Err(_) => return Err(SysError::EINVAL),
        };

        /// check name
        let name_end =  self.salg_name
            .iter().position(|&bytes| bytes == 0)
            .unwrap_or(self.salg_name.len());
        let raw_name = &self.salg_name[..name_end];
        let alg_name = match from_utf8(raw_name) {
            Ok(string) => string,
            Err(_) => return Err(SysError::EINVAL),
        };

        if alg_type == "hash" {
            if alg_name.starts_with("hmac(") && alg_name.ends_with(')') {
                let inner_part = &alg_name[5.. alg_name.len()-1];
                if inner_part.starts_with("hmac(") {
                    return Err(SysError::ENOENT);
                }
            }
        }

        if alg_type == "ahead" {
            if alg_name.starts_with("rfc7539(") && alg_name.ends_with(')') {
                let inner = &alg_name[8.. alg_name.len()-1];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let cipher = parts[0];
                    let mac = parts[1];
                    if cipher == "chacha20" && mac != "poly1305" {
                        return Err(SysError::ENOENT);
                    }
                }else {
                    return Err(SysError::ENOENT);
                }
            }
        }
        Ok(0)

    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AlgType {
    /// Asynchronous Compression
    Acomp,
    /// Authenticated Encryption with Associated Data
    Aead,
    /// Asymmetric Encryption
    Akcipher,
    /// Hash Function
    Hash,
    /// Symmetric Cipher
    Skcipher,
    /// Key Exchange/Derivation
    Kpp, 
    /// Random number generator
    Rng,
    /// Symmetric Comparison
    Scomp,
    /// others
    Unknown(String),
}

impl AlgType {
    pub fn from_bytes(bytes: &[u8; 14]) -> Result<AlgType, SysError> {
        let end = bytes.iter().position(|&byte| byte == 0).unwrap_or(14);
        let string = match from_utf8(&bytes[..end]) {
            Ok(string) => string,
            Err(_) => return Err(SysError::EINVAL),
        };
         match string {
            "hash"     => Ok(AlgType::Hash),       
            "skcipher" => Ok(AlgType::Skcipher),   
            "aead"     => Ok(AlgType::Aead),       
            "rng"      => Ok(AlgType::Rng),        
            "akcipher" => Ok(AlgType::Akcipher),   
            "kpp"      => Ok(AlgType::Kpp),       
            "scomp"    => Ok(AlgType::Scomp),     
            "acomp"    => Ok(AlgType::Acomp),      
            other => {
                Ok(AlgType::Unknown(other.to_string()))
            }
        }
    }
}

/// trait for symmetric cipher, crypto API
/// differnet algroithm may have different context,but all of them should
/// follow the belowing trait
// pub trait CryptoContext: Send +  Sync + DynClone  {
//     fn encrypt(&self, input: &[u8]) -> Result<(), SysError>;
//     fn encrypt_raw(&self, input: &[u8]) -> Result<&[u8], SysError>;
// }

// clone_trait_object!(CryptoContext);
// todo:  different algorithm context

/// AlgInstance structure in socket
#[derive(Clone)]
pub struct AlgInstance {
    pub alg_type: AlgType,
    pub alg_name: String,
    pub alg_key: Vec<u8>,
}

impl AlgInstance {
    pub fn new_without_key(alg_type: AlgType, alg_name: String) -> AlgInstance {
        AlgInstance {
            alg_type,
            alg_name,
            alg_key: Vec::new(),
        }
    }

    pub fn set_key(&mut self, raw_key: &[u8]) {
        self.alg_key = raw_key.to_vec();
    }

    pub fn get_key(&self) -> &[u8] {
        self.alg_key.as_slice()
    }
}

pub fn encode_salsa20(key: &[u8], iv: &[u8], plaintext: &[u8]) -> SockResult<Vec<u8>> {
    let inner_key: [u8; 32]  = match key.len() {
        length if length >= 32 => {
            let mut tmp = [0u8; 32];
            tmp.copy_from_slice(&key[..32]);
            tmp
        },
        16 => {
            let mut tmp: [u8; 32] = [0u8; 32];
            tmp[..16].copy_from_slice(&key[..16]);
            tmp[16..].copy_from_slice(&key[..16]);  // 重复填充
            tmp
        },
        _ => return Err(SysError::EINVAL),
    };
    let iv_slice: &[u8];
    let iv_vec;
    if iv.len() != 8 {
        iv_vec  = vec![0u8; 8];
        iv_slice = &iv_vec;
    }else {
        iv_slice = iv;
    }

    log::warn!("inner key is {:x?}, iv_slice is {:x?}", inner_key, iv_slice);
    let mut cipher = Salsa20::new(&inner_key.into(), iv_slice.into());
    log::warn!("encode salsa20 2");
    let mut ciphertext = plaintext.to_vec();
    cipher.apply_keystream(&mut ciphertext);
    log::warn!("cipher text is {:x?}", ciphertext);
    Ok(ciphertext)
}

pub fn encode_raw(socket: &super::socket::Socket, text: &[u8]) -> SysResult {
     if !socket.get_is_af_alg() {
        log::error!("[crypto/encode_raw]: socket is not af_alg");
        return Err(SysError::EINVAL);
    }
    let socket_alg= socket.socket_af_alg.lock().unwrap();
    let alg_type= AlgType::from_bytes(&socket_alg.salg_type)?;
    let alg_instance = socket.get_alg_instance().ok_or(SysError::EOPNOTSUPP)?; 
    let key = (&alg_instance).get_key();
    log::info!("[crypto/encode_raw] alg_type is {:?}, key is {:x?}", alg_type, key);
    match alg_type {
        AlgType::Hash => {
            let name = socket_alg.get_name()?;
            log::info!("[crypto/encode_raw] hash name is {}", name);
            if name == "vmac64(aes)" {
                // first 16 bytes is aes key, last 16 bytes is polyval key
                let aes_key: &[u8; 16] = key.get(..16)
                    .ok_or(SysError::EINVAL)?
                    .try_into()
                    .map_err(|_| SysError::EINVAL)?;
                log::info!("[crypto/encode_raw] aes_key is {:x?}", aes_key);
                // Polyval key 16 - 32 bytes
                let zero_vec = Vec::new();
                let zero_buf: &[u8] = zero_vec.as_slice();
                let polyval_key: &[u8; 16] = key.get(16..32)
                    .ok_or(SysError::EINVAL)?
                    .try_into()
                    .map_err(|_| SysError::EINVAL)?;
                log::info!("[crypto/encode_raw] polyval_key is {:x?}", polyval_key);
                // set nonce
                let nonce: [u8; 16] = [0u8; 16];
                // count hash(text)
                let mut polyval = Polyval::new(GenericArray::from_slice(polyval_key));
                polyval.update_padded(text);
                let hash_result = polyval.finalize();

                // AES encode nonce
                let aes = Aes128::new(GenericArray::from_slice(aes_key));
                let mut block = GenericArray::clone_from_slice(&nonce);
                aes.encrypt_block(&mut block);
                let aes_output = block;

                // XOR hash
                let mut tag = [0u8; 16];
                for i in 0..16 {
                    tag[i] = hash_result.as_slice()[i] ^ aes_output[i];
                }

                // to 64 bit (VMAC64)
                let truncated = &tag[..8];
                log::error!("[encode_text] ciphertext is {:?}",truncated);
                socket.set_ciphertext(truncated);
            }
            else if name == "sha256" {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(text);
                let result: GenericArray<u8, generic_array::typenum::UInt<generic_array::typenum::UInt<generic_array::typenum::UInt<generic_array::typenum::UInt<generic_array::typenum::UInt<generic_array::typenum::UInt<generic_array::typenum::UTerm, generic_array::typenum::B1>, generic_array::typenum::B0>, generic_array::typenum::B0>, generic_array::typenum::B0>, generic_array::typenum::B0>, generic_array::typenum::B0>> = hasher.finalize();
                socket.set_ciphertext(&result[..]);
            }
            else if name == "sha1" {
                use sha1::Sha1;
                use sha2::Digest;
                let mut hasher = Sha1::new();
                hasher.update(text);
                let result = hasher.finalize();
                socket.set_ciphertext(&result[..]);
            }
            else if name == "hmac(sha256)" {
                // use hmac::{Hmac, Mac};
                // use sha2::Sha256;
                // type HmacSha256 = Hmac<Sha256>;

                // let key = match socket.get_alg_instance() {
                //     Some(alg_instance) => alg_instance.get_key(),
                //     None => return Err(SysError::EOPNOTSUPP),
                // };
                // let mut mac = HmacSha256::new_from_slice(key).map_err(|_| SysError::EINVAL)?;
                // mac.update(text);
                // let result = mac.finalize().into_bytes();
                // socket.set_ciphertext(&result[..]);
                // todo
            }
        },
        AlgType::Skcipher => {
            if socket_alg.get_name()? == "cbc(aes-generic)"{
                if text.len()!=16|| text.len()!=32{
                    return Err(SysError::EINVAL);
                }
                let alg_instance = socket.get_alg_instance().ok_or(SysError::EOPNOTSUPP)?;
                let key = alg_instance.alg_key.as_slice();
                let _aes_key: &[u8; 16] = key.get(..16)
                    .ok_or(SysError::EINVAL)?
                    .try_into()
                    .map_err(|_| SysError::EINVAL)?;
                // todo : implement code concerning iv
            }
        },
        AlgType::Aead => todo!(),
        AlgType::Rng => todo!(),
        AlgType::Akcipher => todo!(),
        AlgType::Kpp => todo!(),
        AlgType::Scomp => todo!(),
        AlgType::Acomp => todo!(),
        AlgType::Unknown(_) => todo!(),
    }
    Ok(0)
}

#[repr(C)]
pub struct Cmsghdr {
    /// length of whole cmsghdr
    pub cmsg_len: u32,
    /// level of protocol
    pub cmsg_level: CmsgLevel,
    /// type of protocol
    pub cmsg_type: CmsgType,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmsgLevel {
    /// SOL_SOCKET 
    SolSocket   = 1,
    /// IPPROTO_IP , ipv4 concerning
    IpProtoIp   = 0,
    /// IPPROTO_IPV6 
    IpProtoIpv6 = 41,
    /// SOL_ALG 
    SolAlg      = 279,
    /// 未知的 level，将保留原始值（如将其存放在其他字段里）
    Unknown     = -1,
}

impl From<i32> for CmsgLevel {
    fn from(raw: i32) -> Self {
        match raw {
            1   => CmsgLevel::SolSocket,
            0   => CmsgLevel::IpProtoIp,
            41  => CmsgLevel::IpProtoIpv6,
            279 => CmsgLevel::SolAlg,
            _   => CmsgLevel::Unknown,
        }
    }
}

impl From<CmsgLevel> for i32 {
    fn from(level: CmsgLevel) -> i32 {
        match level {
            CmsgLevel::SolSocket   => 1,
            CmsgLevel::IpProtoIp   => 0,
            CmsgLevel::IpProtoIpv6 => 41,
            CmsgLevel::SolAlg      => 279,
            CmsgLevel::Unknown     => -1,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmsgTypeSolSocket {
    Rights       = 1,   
    Credentials  = 2,   
    Timestamp    = 29,  
    TimestampNs  = 35,  
    Timestamping = 37,  
    Unknown         = -1,  
}

impl From<i32> for CmsgTypeSolSocket {
    fn from(raw: i32) -> Self {
        match raw {
            1   => CmsgTypeSolSocket::Rights,
            2   => CmsgTypeSolSocket::Credentials,
            29  => CmsgTypeSolSocket::Timestamp,
            35  => CmsgTypeSolSocket::TimestampNs,
            37  => CmsgTypeSolSocket::Timestamping,
            _   => CmsgTypeSolSocket::Unknown,
        }
    }
}

impl From<CmsgTypeSolSocket> for i32 {
    fn from(t: CmsgTypeSolSocket) -> i32 {
        match t {
            CmsgTypeSolSocket::Rights       => 1,
            CmsgTypeSolSocket::Credentials  => 2,
            CmsgTypeSolSocket::Timestamp    => 29,
            CmsgTypeSolSocket::TimestampNs  => 35,
            CmsgTypeSolSocket::Timestamping => 37,
            CmsgTypeSolSocket::Unknown         => -1,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmsgTypeIpProtoIp {
    IpTos      = 1,   
    IpPktInfo  = 8,   
    IpRecvTtl  = 12,  
    IpRecvTos  = 13,
    IpTtl      = 21,   
    Unknown    = -1,  
}

impl From<i32> for CmsgTypeIpProtoIp {
    fn from(raw: i32) -> Self {
        match raw {
            1   => CmsgTypeIpProtoIp::IpTos,
            8   => CmsgTypeIpProtoIp::IpPktInfo,
            12  => CmsgTypeIpProtoIp::IpRecvTtl,
            13  => CmsgTypeIpProtoIp::IpRecvTos, 
            21  => CmsgTypeIpProtoIp::IpTtl,
            _   => CmsgTypeIpProtoIp::Unknown,
        }
    }
}

impl From<CmsgTypeIpProtoIp> for i32 {
    fn from(t: CmsgTypeIpProtoIp) -> i32 {
        match t {
            CmsgTypeIpProtoIp::IpTtl     => 21,
            CmsgTypeIpProtoIp::IpTos     => 1,
            CmsgTypeIpProtoIp::IpPktInfo => 8,
            CmsgTypeIpProtoIp::IpRecvTtl => 12,
            CmsgTypeIpProtoIp::IpRecvTos => 13,
            CmsgTypeIpProtoIp::Unknown   => -1,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmsgTypeIpProtoIpv6 {
    Ipv6HopLimit     = 52,
    Ipv6Tclass       = 67,
    Ipv6PktInfo      = 50,
    Ipv6HopOpts      = 54,
    Ipv6RtHdr        = 51,
    Ipv6DstOpts      = 58,
    Ipv6RecvPktInfo  = 49,
    Ipv6RecvHopOpts  = 56,
    Ipv6RecvDstOpts  = 57,
    Ipv6RecvRtHdr    = 55,
    Unknown          = -1,
}

impl From<i32> for CmsgTypeIpProtoIpv6 {
    fn from(raw: i32) -> Self {
        match raw {
            52  => CmsgTypeIpProtoIpv6::Ipv6HopLimit,
            67  => CmsgTypeIpProtoIpv6::Ipv6Tclass,
            50  => CmsgTypeIpProtoIpv6::Ipv6PktInfo,
            54  => CmsgTypeIpProtoIpv6::Ipv6HopOpts,
            51  => CmsgTypeIpProtoIpv6::Ipv6RtHdr,
            58  => CmsgTypeIpProtoIpv6::Ipv6DstOpts,
            49  => CmsgTypeIpProtoIpv6::Ipv6RecvPktInfo, 
            56  => CmsgTypeIpProtoIpv6::Ipv6RecvHopOpts,
            57  => CmsgTypeIpProtoIpv6::Ipv6RecvDstOpts,
            55  => CmsgTypeIpProtoIpv6::Ipv6RecvRtHdr,
            _   => CmsgTypeIpProtoIpv6::Unknown,
        }
    }
}

impl From<CmsgTypeIpProtoIpv6> for i32 {
    fn from(t: CmsgTypeIpProtoIpv6) -> i32 {
        match t {
            CmsgTypeIpProtoIpv6::Ipv6HopLimit     => 52,
            CmsgTypeIpProtoIpv6::Ipv6Tclass       => 67,
            CmsgTypeIpProtoIpv6::Ipv6PktInfo      => 50,
            CmsgTypeIpProtoIpv6::Ipv6HopOpts      => 54,
            CmsgTypeIpProtoIpv6::Ipv6RtHdr        => 51,
            CmsgTypeIpProtoIpv6::Ipv6DstOpts      => 58,
            CmsgTypeIpProtoIpv6::Ipv6RecvPktInfo => 49,
            CmsgTypeIpProtoIpv6::Ipv6RecvHopOpts  => 56,
            CmsgTypeIpProtoIpv6::Ipv6RecvDstOpts  => 57,
            CmsgTypeIpProtoIpv6::Ipv6RecvRtHdr    => 55,
            CmsgTypeIpProtoIpv6::Unknown          => -1,
        }
    }
}

/// (4) SOL_ALG (level=279) 下的 cmsg_type
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmsgTypeSolAlg {
    AlgSetOp           = 3,
    AlgSetIv           = 2,
    AlgSetAeadAssoc    = 4,
    AlgSetAeadAuthSize = 5,
    Unknown            = -1,
}

impl From<i32> for CmsgTypeSolAlg {
    fn from(raw: i32) -> Self {
        match raw {
            3 => CmsgTypeSolAlg::AlgSetOp,
            2 => CmsgTypeSolAlg::AlgSetIv,
            4 => CmsgTypeSolAlg::AlgSetAeadAssoc,
            5 => CmsgTypeSolAlg::AlgSetAeadAuthSize,
            _ => CmsgTypeSolAlg::Unknown,
        }
    }
}

impl From<CmsgTypeSolAlg> for i32 {
    fn from(t: CmsgTypeSolAlg) -> i32 {
        match t {
            CmsgTypeSolAlg::AlgSetOp           => 3,
            CmsgTypeSolAlg::AlgSetIv           => 2,
            CmsgTypeSolAlg::AlgSetAeadAssoc    => 4,
            CmsgTypeSolAlg::AlgSetAeadAuthSize => 5,
            CmsgTypeSolAlg::Unknown            => -1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmsgType {
    SolSocket(CmsgTypeSolSocket),
    ///
    IpProtoIp(CmsgTypeIpProtoIp),
    /// 
    IpProtoIpv6(CmsgTypeIpProtoIpv6),
    /// 
    SolAlg(CmsgTypeSolAlg),
    /// Unknown type
    Unknown { level: CmsgLevel, raw: i32 },
}

impl CmsgType {
    pub fn parse(level_raw: i32, type_raw: i32) -> Self {
        let level = CmsgLevel::from(level_raw);
        match level {
            CmsgLevel::SolSocket => {
                let t = CmsgTypeSolSocket::from(type_raw);
                CmsgType::SolSocket(t)
            }
            CmsgLevel::IpProtoIp => {
                let t = CmsgTypeIpProtoIp::from(type_raw);
                CmsgType::IpProtoIp(t)
            }
            CmsgLevel::IpProtoIpv6 => {
                let t = CmsgTypeIpProtoIpv6::from(type_raw);
                CmsgType::IpProtoIpv6(t)
            }
            CmsgLevel::SolAlg => {
                let t = CmsgTypeSolAlg::from(type_raw);
                CmsgType::SolAlg(t)
            }
            CmsgLevel::Unknown => {
                CmsgType::Unknown { level, raw: type_raw }
            }
        }
    }
}

pub fn encode(socket: &Arc<super::socket::Socket>, _iovec: &[IoVec], control:&[u8]) -> SysResult {
    fn cmsghdr_from_bytes(bytes: &[u8]) -> SockResult<(Cmsghdr, &[u8])> {
        if bytes.len() < 16 {
            return Err(SysError::EFAULT);
        }

        /// read cmsg_len
        let mut offset = 0;
        let mut buf = [0u8; size_of::<usize>()];
        buf.copy_from_slice(&bytes[offset..offset + size_of::<usize>()]);
        offset += size_of::<usize>();
        let cmsg_len = usize::from_ne_bytes(buf);

        /// read cmsg_level
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&bytes[offset..offset + 4]);
        offset += 4;
        let raw_level = i32::from_ne_bytes(buf);
        let cmsg_level = CmsgLevel::from(raw_level);

        /// read cmsg_type
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&bytes[offset..offset + 4]);
        offset += 4;
        let raw_type = i32::from_ne_bytes(buf);
        let cmsg_type = CmsgType::parse(raw_level, raw_type);
        
        let cmsghdr = Cmsghdr {
            cmsg_len: cmsg_len as u32,
            cmsg_level,
            cmsg_type,
        };

        if cmsg_level != CmsgLevel::SolAlg {
            return Ok((cmsghdr, &[]));
        }

        if let CmsgType::SolAlg(alg) = cmsghdr.cmsg_type {
            return Ok((cmsghdr, &[]));
        }

       let iv_offset = size_of::<Cmsghdr>();
       if (cmsghdr.cmsg_len as usize) < iv_offset + size_of::<u32>() {
           return Err(SysError::EINVAL);
       }

       if bytes.len() < iv_offset as usize + size_of::<u32>() {
           return Err(SysError::EFAULT);
       }

       let start = iv_offset;
       let end = start + size_of::<u32>();
       let mut buf = [0u8; size_of::<u32>()];
       buf.copy_from_slice(&bytes[start..end]);
       let ivlen = u32::from_ne_bytes(buf) as usize;

       let iv_start = iv_offset + size_of::<u32>();
       let iv_end = iv_start + ivlen;
       let iv = &bytes[iv_start..iv_end];

       if cmsghdr.cmsg_len < iv_end as u32 {
            return Err(SysError::EINVAL);
       }
       if bytes.len() <  iv_end {
           return Err(SysError::EFAULT);
       }
       Ok((cmsghdr, iv))

    } 
    
    let inner_socket_alg = socket.socket_af_alg.lock().unwrap();
    let alg_type = AlgType::from_bytes(&inner_socket_alg.salg_type)?;
    let (cmsg, iv) = cmsghdr_from_bytes(&control)?;

    match alg_type {
        AlgType::Skcipher => {
            if inner_socket_alg.get_name()? == "salsa20" {
                let alg_instance = socket.get_alg_instance().unwrap();
                let key = alg_instance.get_key();
                let ciphertext = encode_salsa20(key, &iv, Vec::new().as_slice())?;
                socket.set_ciphertext(ciphertext.as_slice());
                return Ok(ciphertext.len() as isize);
            }else {
                return Ok(0);
            }
        },
        _ => {
            //todo
            return Ok(0);
        }
    }
}
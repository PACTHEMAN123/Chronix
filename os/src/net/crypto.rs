use core::str::from_utf8;

use alloc::{boxed::Box, string::{String, ToString}, vec::Vec};

use crate::syscall::SysError;

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
}
#[derive(Debug, PartialEq, Eq)]
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
                // 其余的不在上述列表里的，全部归入 Unknown
                Ok(AlgType::Unknown(other.to_string()))
            }
        }
    }
}

/// trait for symmetric cipher, crypto API
/// differnet algroithm may have different context,but all of them should
/// follow the belowing trait
pub trait CryptoContext: Send +  Sync  {
    fn set_key(&mut self, key: &[u8]) -> Result<(), SysError>;
    fn encrypt(&self, input: &[u8]) -> Result<(), SysError>;
    fn decrypt(&self, input: &[u8]) -> Result<(), SysError>;
}

// todo:  different algorithm context

/// AlgInstance structure in socket
pub struct AlgInstance {
    pub alg_type: AlgType,
    pub context: Box<dyn CryptoContext>,
}
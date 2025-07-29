use core::str::from_utf8;

use alloc::{boxed::Box, string::{String, ToString}, vec::Vec};

use dyn_clone::{clone_trait_object, DynClone};
use crate::{net::socket::SockResult, syscall::SysError};

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
                // 其余的不在上述列表里的，全部归入 Unknown
                Ok(AlgType::Unknown(other.to_string()))
            }
        }
    }
}

/// trait for symmetric cipher, crypto API
/// differnet algroithm may have different context,but all of them should
/// follow the belowing trait
pub trait CryptoContext: Send +  Sync + DynClone  {
    fn set_key(&mut self, key: &[u8]) -> Result<(), SysError>;
    fn encrypt(&self, input: &[u8]) -> Result<(), SysError>;
    fn decrypt(&self, input: &[u8]) -> Result<(), SysError>;
}

clone_trait_object!(CryptoContext);
// todo:  different algorithm context

/// AlgInstance structure in socket
#[derive(Clone)]
pub struct AlgInstance {
    pub alg_type: AlgType,
    pub context: Box<dyn CryptoContext>,
}
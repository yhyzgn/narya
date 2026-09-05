use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

pub const PROTOCOL_VERSION: u16 = 1;
pub const FRAME_HEADER_LEN: usize = 4;
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IpcRequest {
    #[serde(default = "default_protocol_version")]
    pub version: u16,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct IpcResponse {
    #[serde(default = "default_protocol_version")]
    pub version: u16,
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct KernelInfo {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub running: bool,
    pub healthy: bool,
    pub state: String,
    pub failure: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum IpcNotification {
    TrafficUpdate { down: f32, up: f32 },
    StatusUpdate { running: bool },
    LogLine { level: String, message: String },
    KernelStatusUpdate { kernels: Vec<KernelInfo> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    TooLarge(usize),
    Truncated,
    InvalidJson(String),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge(size) => write!(f, "IPC frame exceeds maximum size: {size} bytes"),
            Self::Truncated => write!(f, "IPC frame is truncated"),
            Self::InvalidJson(error) => write!(f, "invalid IPC JSON: {error}"),
        }
    }
}

impl std::error::Error for FrameError {}

pub fn encode_frame<T: Serialize>(message: &T) -> Result<Vec<u8>, FrameError> {
    let payload =
        serde_json::to_vec(message).map_err(|error| FrameError::InvalidJson(error.to_string()))?;
    encode_payload(&payload)
}

fn default_protocol_version() -> u16 {
    PROTOCOL_VERSION
}

pub fn decode_frame<T: DeserializeOwned>(payload: &[u8]) -> Result<T, FrameError> {
    serde_json::from_slice(payload).map_err(|error| FrameError::InvalidJson(error.to_string()))
}

pub fn encode_payload(payload: &[u8]) -> Result<Vec<u8>, FrameError> {
    if payload.len() > MAX_FRAME_SIZE {
        return Err(FrameError::TooLarge(payload.len()));
    }
    let length = u32::try_from(payload.len()).map_err(|_| FrameError::TooLarge(payload.len()))?;
    let mut frame = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(payload);
    Ok(frame)
}

#[derive(Debug, Default)]
pub struct FrameDecoder {
    buffer: Vec<u8>,
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, FrameError> {
        self.buffer.extend_from_slice(bytes);
        let mut frames = Vec::new();
        loop {
            if self.buffer.len() < FRAME_HEADER_LEN {
                break;
            }
            let size = u32::from_be_bytes([
                self.buffer[0],
                self.buffer[1],
                self.buffer[2],
                self.buffer[3],
            ]) as usize;
            if size > MAX_FRAME_SIZE {
                self.buffer.clear();
                return Err(FrameError::TooLarge(size));
            }
            let frame_len = FRAME_HEADER_LEN + size;
            if self.buffer.len() < frame_len {
                break;
            }
            let payload = self.buffer[FRAME_HEADER_LEN..frame_len].to_vec();
            self.buffer.drain(..frame_len);
            frames.push(payload);
        }
        Ok(frames)
    }

    pub fn finish(self) -> Result<(), FrameError> {
        if self.buffer.is_empty() {
            Ok(())
        } else {
            Err(FrameError::Truncated)
        }
    }

    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }
}

pub fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("TMPDIR").map(|tmp| {
                let mut path = PathBuf::from(tmp);
                path.push(format!("narya-{}", runtime_owner()));
                path
            })
        })
        .unwrap_or_else(|| {
            let mut path = std::env::temp_dir();
            path.push(format!("narya-{}", runtime_owner()));
            path
        })
        .join("narya")
}

fn runtime_owner() -> String {
    std::env::var("UID")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "current".to_string())
}

pub fn socket_path() -> PathBuf {
    let name = std::env::var("NARYA_SOCKET_NAME")
        .ok()
        .filter(|name| {
            !name.is_empty()
                && name.len() <= 120
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        })
        .unwrap_or_else(|| "narya.sock".into());
    runtime_dir().join(name)
}

pub fn kernel_config_path() -> PathBuf {
    runtime_dir().join("kernel.json")
}

pub fn kernel_install_dir() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| std::env::temp_dir().join("narya-data"));
    base.join("narya").join("kernels")
}

pub fn ruleset_cache_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(|| std::env::temp_dir().join("narya-cache"));
    base.join("narya").join("rulesets")
}

pub fn kernel_catalog_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| std::env::temp_dir().join("narya-config"));
    base.join("narya").join("kernel-catalog")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: u64) -> IpcRequest {
        IpcRequest {
            version: PROTOCOL_VERSION,
            id,
            method: "Ping".into(),
            params: serde_json::json!({"id": id}),
        }
    }

    #[test]
    fn decoder_handles_one_byte_chunks() {
        let frame = encode_frame(&request(7)).unwrap();
        let mut decoder = FrameDecoder::new();
        let mut decoded = Vec::new();
        for byte in frame {
            decoded.extend(decoder.push(&[byte]).unwrap());
        }
        assert_eq!(decoded.len(), 1);
        assert_eq!(decode_frame::<IpcRequest>(&decoded[0]).unwrap(), request(7));
        assert_eq!(decoder.buffered_len(), 0);
    }

    #[test]
    fn decoder_handles_concatenated_frames() {
        let mut bytes = encode_frame(&request(1)).unwrap();
        bytes.extend(encode_frame(&request(2)).unwrap());
        let mut decoder = FrameDecoder::new();
        let decoded = decoder.push(&bytes).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decode_frame::<IpcRequest>(&decoded[0]).unwrap(), request(1));
        assert_eq!(decode_frame::<IpcRequest>(&decoded[1]).unwrap(), request(2));
    }

    #[test]
    fn decoder_rejects_oversized_and_truncated_frames() {
        let oversized = (MAX_FRAME_SIZE as u32 + 1).to_be_bytes();
        assert!(matches!(
            FrameDecoder::new().push(&oversized),
            Err(FrameError::TooLarge(_))
        ));
        let frame = encode_frame(&request(3)).unwrap();
        let mut decoder = FrameDecoder::new();
        decoder.push(&frame[..frame.len() - 1]).unwrap();
        assert!(matches!(decoder.finish(), Err(FrameError::Truncated)));
    }
}

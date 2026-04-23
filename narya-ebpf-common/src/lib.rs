#![no_std]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrafficStats {
    pub packets: u64,
    pub bytes: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ProcessConfig {
    pub action: u32, // 0: Pass/Direct, 1: Proxy/Redirect, 2: Drop/Reject
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for TrafficStats {}
#[cfg(feature = "user")]
unsafe impl aya::Pod for ProcessConfig {}

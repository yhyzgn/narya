#![no_std]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrafficStats {
    pub packets: u64,
    pub bytes: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessConfig {
    pub action: u32, // 0: Direct, 1: Proxy, 2: Reject
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for TrafficStats {}
#[cfg(feature = "user")]
unsafe impl aya::Pod for ProcessConfig {}

#![cfg_attr(target_arch = "bpf", no_std)]
#![cfg_attr(target_arch = "bpf", no_main)]

#[cfg(target_arch = "bpf")]
use aya_ebpf::{
    helpers::bpf_get_current_pid_tgid,
    macros::{cgroup_skb, map},
    maps::HashMap,
    programs::SkBuffContext,
};
#[cfg(target_arch = "bpf")]
use narya_ebpf_common::{ProcessConfig, TrafficStats};

#[cfg(target_arch = "bpf")]
#[map(name = "TRAFFIC_STATS")]
static mut TRAFFIC_STATS: HashMap<u32, TrafficStats> =
    HashMap::<u32, TrafficStats>::with_max_entries(1024, 0);

#[cfg(target_arch = "bpf")]
#[map(name = "PROCESS_RULES")]
static mut PROCESS_RULES: HashMap<u32, ProcessConfig> =
    HashMap::<u32, ProcessConfig>::with_max_entries(1024, 0);

#[cfg(target_arch = "bpf")]
#[cgroup_skb(name = "narya_egress")]
pub fn narya_egress(ctx: SkBuffContext) -> i32 {
    match try_narya_egress(ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

#[cfg(target_arch = "bpf")]
fn try_narya_egress(ctx: SkBuffContext) -> Result<i32, u32> {
    let skb = ctx.skb;
    let len = unsafe { (*skb).len as u64 };

    // 获取当前进程的 PID
    let pid = (unsafe { bpf_get_current_pid_tgid() } >> 32) as u32;

    // 1. 流量统计
    unsafe {
        if let Some(stats) = TRAFFIC_STATS.get_ptr_mut(&pid) {
            (*stats).packets += 1;
            (*stats).bytes += len;
        } else {
            let new_stats = TrafficStats {
                packets: 1,
                bytes: len,
            };
            let _ = TRAFFIC_STATS.insert(&pid, &new_stats, 0);
        }
    }

    // 2. 规则匹配
    unsafe {
        if let Some(config) = PROCESS_RULES.get(&pid) {
            match config.action {
                2 => return Ok(0), // 0 表示 Drop (Reject)
                _ => return Ok(1), // 1 表示 Pass
            }
        }
    }

    Ok(1)
}

#[cfg(target_arch = "bpf")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

#[cfg(not(target_arch = "bpf"))]
fn main() {}

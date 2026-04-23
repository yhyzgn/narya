#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{cgroup_skb, map},
    maps::HashMap,
    programs::SkBuffContext,
};
use narya_ebpf_common::TrafficStats;

#[map(name = "TRAFFIC_STATS")]
static mut TRAFFIC_STATS: HashMap<u32, TrafficStats> =
    HashMap::<u32, TrafficStats>::with_max_entries(1024, 0);

#[cgroup_skb(name = "narya_egress")]
pub fn narya_egress(ctx: SkBuffContext) -> i32 {
    #[cfg(target_arch = "bpf")]
    {
        match try_narya_egress(ctx) {
            Ok(ret) => ret,
            Err(_) => 1,
        }
    }
    #[cfg(not(target_arch = "bpf"))]
    {
        let _ = ctx;
        1
    }
}

#[cfg(target_arch = "bpf")]
fn try_narya_egress(ctx: SkBuffContext) -> Result<i32, u32> {
    let skb = ctx.skb;
    let len = unsafe { (*skb).len as u64 };

    // 获取当前进程的 cgroup id 或 pid
    let key = 0u32;

    unsafe {
        if let Some(stats) = TRAFFIC_STATS.get_ptr_mut(&key) {
            (*stats).packets += 1;
            (*stats).bytes += len;
        } else {
            let new_stats = TrafficStats {
                packets: 1,
                bytes: len,
            };
            let _ = TRAFFIC_STATS.insert(&key, &new_stats, 0);
        }
    }

    Ok(1)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

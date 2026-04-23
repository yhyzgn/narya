#![cfg_attr(target_arch = "bpf", no_std)]
#![cfg_attr(target_arch = "bpf", no_main)]

#[cfg(target_arch = "bpf")]
use aya_ebpf::{
    macros::{cgroup_skb, map},
    maps::HashMap,
    programs::SkBuffContext,
};
#[cfg(target_arch = "bpf")]
use narya_ebpf_common::TrafficStats;

#[cfg(target_arch = "bpf")]
#[map(name = "TRAFFIC_STATS")]
static mut TRAFFIC_STATS: HashMap<u32, TrafficStats> =
    HashMap::<u32, TrafficStats>::with_max_entries(1024, 0);

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

#[cfg(target_arch = "bpf")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

#[cfg(not(target_arch = "bpf"))]
fn main() {
    // 宿主机环境下变成一个什么都不做的普通程序，防止编译报错
}

#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{cgroup_skb, map},
    maps::HashMap,
    programs::SkBuffContext,
};
use aya_log_ebpf::info;
use narya_ebpf_common::TrafficStats;

#[map(name = "TRAFFIC_STATS")]
static mut TRAFFIC_STATS: HashMap<u32, TrafficStats> =
    HashMap::<u32, TrafficStats>::with_max_entries(1024, 0);

#[cgroup_skb(name = "narya_egress")]
pub fn narya_egress(ctx: SkBuffContext) -> i32 {
    match try_narya_egress(ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

fn try_narya_egress(ctx: SkBuffContext) -> Result<i32, u32> {
    let skb = ctx.skb;
    let len = unsafe { (*skb).len as u64 };

    // 获取当前进程的 cgroup id 或 pid (在 cgroup_skb 中通常通过 ctx 获取)
    // 简化版：使用 0 作为全局统计，实际开发中会根据 cgroup 划分
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

    Ok(1) // 1 表示放行 (Pass)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

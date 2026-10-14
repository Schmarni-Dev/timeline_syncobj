use std::{sync::Arc, thread::sleep, time::Duration};

use timeline_syncobj::{render_node::DrmRenderNode, timeline_syncobj::TimelineSyncObj};

fn main() {
    let node = DrmRenderNode::new(128).unwrap();
    let timeline = Arc::new(TimelineSyncObj::new(&node).unwrap());
    let fd = timeline.export().unwrap();
    println!("{fd:?}");
    let timeline2 = timeline.clone();
    std::thread::spawn(move || {
        sleep(Duration::from_millis(200 as u64));
        unsafe {
            timeline2.signal(8).unwrap();
        };
    });
    timeline.blocking_wait(8, None).unwrap();
    // unsafe {
    //     timeline.signal(8).unwrap();
    // };
    println!("returned from wait");
    let fd = timeline.export_sync_file_point(1).unwrap();
    println!("{fd:?}");
}

use std::{sync::Arc, thread::sleep, time::Duration};

use timeline_syncobj::{render_node::DrmRenderNode, timeline_syncobj::TimelineSyncObj};

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
    point_signaling();
    point_transfer();
    sharing();
    sync_file_exporting();
    // async_wait().await;
}
fn point_signaling() {
    let node = DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    assert_eq!(obj.query_signaled_point(), Ok(0));
    unsafe { obj.signal(32).unwrap() };
    assert_eq!(obj.query_signaled_point(), Ok(32));
}

fn point_transfer() {
    let node = DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    let obj2 = TimelineSyncObj::new(&node).expect("failed to create second syncojb");
    assert_eq!(obj2.query_signaled_point(), Ok(0));
    unsafe { obj.signal(32).unwrap() };
    obj.transfer_point(&obj2, 32, 1).unwrap();
    assert_eq!(obj.query_signaled_point(), Ok(32));
    assert_eq!(obj2.query_signaled_point(), Ok(1));
}

fn sharing() {
    let node = DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    let obj_fd = obj.export().expect("failed to export syncobj");
    let obj2 = TimelineSyncObj::import(&node, obj_fd).expect("failed to import syncobj");
    assert_eq!(obj.query_signaled_point(), Ok(0));
    assert_eq!(obj2.query_signaled_point(), Ok(0));
    unsafe { obj.signal(32).unwrap() };
    assert_eq!(obj.query_signaled_point(), Ok(32));
    assert_eq!(obj2.query_signaled_point(), Ok(32));
}

fn sync_file_exporting() {
    use std::io::ErrorKind;
    let node = DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    let obj2 = TimelineSyncObj::new(&node).expect("failed to create second syncobj");
    assert!(
        obj.export_sync_file_point(32)
            .is_err_and(|err| err.kind() == ErrorKind::InvalidInput)
    );
    unsafe { obj.signal(32).unwrap() };
    let export = obj.export_sync_file_point(32).unwrap();
    obj2.import_sync_file_point(export, 32).unwrap();
    assert_eq!(obj2.query_signaled_point(), Ok(32));
}
// async fn async_wait() {
//     let node = DrmRenderNode::new(128).expect("failed to open render node");
//     let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
//     let wait = tokio::spawn(obj.wait_async(16).unwrap());
//     tokio::time::sleep(std::time::Duration::from_millis(1)).await;
//     assert!(!wait.is_finished());
//     unsafe { obj.signal(32).unwrap() };
//     tokio::time::sleep(std::time::Duration::from_millis(1)).await;
//     assert!(wait.is_finished());
//     wait.await.unwrap();
// }

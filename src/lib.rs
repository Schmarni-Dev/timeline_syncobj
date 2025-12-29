pub mod bindings;
pub mod timeline_syncobj;
pub mod render_node;
#[cfg(feature = "tokio")]
pub mod tokio_integration;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

use crate::cache::CACHE_DIR;

#[test]
fn test_cache_dir_obtainable() {
    CACHE_DIR.clone().unwrap();
}

#[test]
fn default_config_works() {
    crate::Config::try_default().unwrap();
}

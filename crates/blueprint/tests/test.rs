use blueprint_macros::bake_toml;

bake_toml! {
    "tests/input.toml",
    u32,
    a: 10,
    b: 5,
    c: 1,
}

#[test]
fn test_entry_getters() {
    assert_eq!(ENTRIES[0].a(), 7);
    assert_eq!(ENTRIES[0].b(), 2);
    assert_eq!(ENTRIES[0].c(), 1);
    assert_eq!(ENTRIES[1].a(), 900);
    assert_eq!(ENTRIES[1].b(), 31);
    assert_eq!(ENTRIES[1].c(), 0);
}

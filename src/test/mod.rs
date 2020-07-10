#[test]
fn test_js_minification() {
    assert_eq!(super::esbuild("let a = 1;"), "let a=1;\n");
}

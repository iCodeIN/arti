#[test]
fn apply_simple() {
    let pre = include_str!("../testdata/consensus1.txt");
    let diff = include_str!("../testdata/diff-modified.txt");
    let post = include_str!("../testdata/consensus2.txt");

    let result = tor_consdiff::apply_diff(pre, diff).unwrap();
    assert_eq!(result.to_string(), post);
}
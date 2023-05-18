mod testutil;
use testutil::testutil::build_regular_hprtree;

#[cfg(feature = "svgexport")]
#[test]
fn svgexport_test() {
    let tree = build_regular_hprtree(1);
    tree.export_svg("target/svgexport_test.svg".to_string());
}
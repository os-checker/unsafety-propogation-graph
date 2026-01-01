#![feature(rustc_private)]

extern crate compiletest_rs as compiletest;

use std::{env, path::PathBuf, sync::LazyLock};

fn run_mode(dir: &str) {
    let bless = env::var("BLESS").is_ok_and(|x| !x.trim().is_empty());
    let dir = &format!("tests/{dir}");

    let config = compiletest::Config {
        bless,
        mode: compiletest::common::Mode::Ui,
        edition: Some("2015".into()),
        src_base: PathBuf::from(dir),
        build_base: PROFILE_PATH.join(dir),
        rustc_path: PROFILE_PATH.join(RUSTC_DRIVER),
        target_rustcflags: Some("--crate-type=lib".to_owned()),
        ..Default::default()
    };

    // config.link_deps(); // Populate config.target_rustcflags with dependencies on the path
    // config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

    compiletest::run_tests(&config);
}

const RUSTC_DRIVER: &str = "unsafety-propogation-graph";

static PROFILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let current_exe_path = env::current_exe().unwrap();
    let deps_path = current_exe_path.parent().unwrap();
    let profile_path = deps_path.parent().unwrap();
    profile_path.into()
});

#[test]
fn compile_test() {
    run_mode("pass");
}

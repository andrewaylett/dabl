use dbus_codegen::GenOpts;
use std::env;
use std::fs;
use std::path::Path;

const PREFIX: &str = "OrgFreedesktop";
const COMMAND_LINE: &str = "gdbus introspect --system --dest org.freedesktop.resolve1 --object-path /org/freedesktop/resolve1 --xml";

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("resolve1.rs");
    // gdbus introspect --system --dest org.freedesktop.resolve1 --object-path /org/freedesktop/resolve1 --xml
    let resolve = include_str!("src/resolve1.xml");
    let opts = GenOpts {
        methodtype: None,
        crhandler: None,
        skipprefix: Some(PREFIX.to_owned()),
        command_line: COMMAND_LINE.to_owned(),
        ..Default::default()
    };
    let output = dbus_codegen::generate(resolve, &opts).expect("CodeGen Failed");
    fs::write(&dest_path, output).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resolve1.xml");
}

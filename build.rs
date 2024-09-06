use std::{fs::{create_dir, remove_dir_all}, process::Command};
use fs_extra::{copy_items, dir::CopyOptions};

fn main() {
    let mut command = Command::new("npm.cmd");
    command.args(["run", "build", "--prefix", "budgeting-ui"]);

    println!("Building ui `{:?}`", command);
    let output = command.output()
        .expect("npm build to succeed");

    if output.status.success() {
        // todo: this is apparently illegal. maybe have it so it's just when running locally
        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR var to be set");
        let out_dir = format!("{out_dir}/../../..");

        let out_dir_dist = format!("{out_dir}/dist");

        remove_dir_all(&out_dir_dist).unwrap();

        create_dir(&out_dir_dist)
            .expect("create dir to succeed");

        copy_items(
            &["budgeting-ui/dist"],
             format!("{out_dir}"),
             &CopyOptions::new()).expect("copy to succeed");

        // success
        return;
    }

    let output_str = String::from_utf8(output.stdout);

    panic!("{}", output_str.unwrap_or("<No Output>".to_owned()));
}
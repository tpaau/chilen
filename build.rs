use std::process::Command;

fn main() {
	let output = Command::new("git")
		.args(["rev-parse", "--short", "HEAD"])
		.output()
		.expect("failed to run git");

	assert!(output.status.success(), "git rev-parse failed");

	let commit = String::from_utf8(output.stdout).expect("invalid UTF-8");
	println!("cargo:rustc-env=APP_GIT_COMMIT={}", commit.trim());
}


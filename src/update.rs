use self_update::cargo_crate_version;
use std::{sync::mpsc::Sender, thread};

fn gh_update() -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(not(target_os = "linux"))]
    let target = "";
    #[cfg(target_os = "linux")]
    let target = "_linux";
    #[cfg(target_os = "macos")]
    let target = "_mac";

    let status = self_update::backends::github::Update::configure()
        .repo_owner("woelper")
        .repo_name("oculante")
        .bin_name("oculante")
        .target(target)
        .current_version(cargo_crate_version!())
        .no_confirm(true)
        .build()?
        .update()?;
    println!("Update status: `{}`!", status.version());
    Ok(format!("{:?}", status))
}

pub fn update(sender: Option<Sender<String>>) {
    thread::spawn(move || match gh_update() {
        Ok(res) => {
            let _ = sender.map(|s| s.send(res));
        }
        Err(e) => {
            let _ = sender.map(|s| s.send(format!("{:?}", e)));
        }
    });
}

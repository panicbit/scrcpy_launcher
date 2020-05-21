use std::collections::BTreeMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const SCRCPY_DELAY: Duration = Duration::from_millis(1_000);
const SCRCPY_ARGS: &[&str] = &[
    "-b", "4M",
    "-m", "1024",
    "-S",
];

fn main() {
    let mut monitor = udev::MonitorBuilder::new().unwrap()
        .match_subsystem("usb").unwrap()
        .listen()
        .unwrap();

    loop {
        let event = match monitor.next() {
            Some(event) => event,
            None => {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
        };

        let properties = event
            .properties()
            .map(|prop| (
                prop.name().to_string_lossy().into_owned(),
                prop.value().to_string_lossy().into_owned(),
            ))
            .collect::<BTreeMap<_, _>>();

        let is_add_action = properties.get("ACTION").map(<_>::as_ref) == Some("add");

        let is_adb = properties.get("DEVLINKS")
            .map(<_>::as_ref)
            .unwrap_or("")
            .contains("android_adb");

        if !is_add_action || !is_adb {
            continue;
        }

        thread::sleep(SCRCPY_DELAY);

        let res = Command::new("scrcpy")
            .args(SCRCPY_ARGS)
            .spawn();

        if let Err(err) = res {
            println!("Error launching srcpy: {}", err);
        }
    }
}

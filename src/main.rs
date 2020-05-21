use std::collections::BTreeMap;
use std::os::unix::io::AsRawFd;
use std::process::Command;
use std::thread;
use std::time::Duration;
use mio::{Events, Poll};
use mio::{Token, unix::SourceFd, Interest};

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

    let monitor_fd = monitor.as_raw_fd();
    let mut monitor_fd = SourceFd(&monitor_fd);
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    poll.registry().register(
        &mut monitor_fd,
        Token(0),
        Interest::READABLE,
    ).unwrap();

    loop {
        poll.poll(&mut events, None).unwrap();
        
        eprintln!("Incoming events!");

        for event in &mut monitor {
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
                eprintln!("Error launching srcpy: {}", err);
            }
        }
    }
}

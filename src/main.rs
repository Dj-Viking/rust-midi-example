extern crate portmidi as pm;

use std::thread;
use std::io;
use std::time::Duration;
use std::os::raw::c_int;

fn get_xonek2_id(pm: &pm::PortMidi) -> c_int {
	let mut ret = 0;
    for d in pm.devices().unwrap() {
		if d.name().contains("XONE") {
			ret = d.id();
		} else { ret = 0; }
    }
	ret
}

static mut INTENSITY: u8 = 0;
const TIMEOUT: Duration = Duration::from_millis(10);

#[derive(Debug)]
struct MyMidiMessage {
	channel: u8,
	intensity: u8,
}
impl MyMidiMessage {
	fn new(m: pm::types::MidiEvent) -> Self {
		Self {
			channel: m.message.data1,
			intensity: m.message.data2,
		}
	}
}

fn handle_midi_msg(m: MyMidiMessage) -> () {
	println!("{:?}", m);

	unsafe {
		INTENSITY = m.intensity;
		println!("{}", INTENSITY);
	}
}

fn main() {
	
	let pm_context = pm::PortMidi::new().unwrap();
	let xone_id = get_xonek2_id(&pm_context);
	// get the device info for the given id
	let info = pm_context.device(xone_id).unwrap();

	println!("Listening on: {}) {}", info.id(), info.name());
	println!("found xone {}", xone_id);

	thread::spawn(move || {
		// get the device's input port
		let in_port = pm_context.input_port(info, 1024).unwrap();

		while let Ok(_) = in_port.poll() {
			if let Ok(Some(m)) = in_port.read_n(1024) {
				handle_midi_msg(MyMidiMessage::new(m[0]));
			}
			// there is no blocking receive method in PortMidi, therefore
			// we have to sleep some time to prevent a busy-wait loop
			thread::sleep(TIMEOUT);
		}
	});

	println!("press enter to quit");
	let mut user_input = String::new();
	io::stdin().read_line(&mut user_input).ok();
}

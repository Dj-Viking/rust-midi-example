use portmidi as pm;

use std::thread;
use std::io;
use std::time::Duration;
use std::os::raw::c_int;

// takes midi messages outputted from wine Ableton midi track out
// PipeWire-RT-Event for MIDI to routing out of ableton
fn get_wine_alsa_input_id(pm: &pm::PortMidi) -> c_int {
	let mut ret = 0;

	for d in pm.devices().unwrap() {
		if d.name().contains("WINE ALSA Output #1") {
			ret = d.id();
		}
	}

	ret
}
fn get_midithroughport_id(pm: &pm::PortMidi) -> c_int {
	let mut ret = 0;

	for d in pm.devices().unwrap() {
		if d.name().contains("Through Port") {
			ret = d.id();
		}
	}

	ret
}
fn get_xonek2_id(pm: &pm::PortMidi) -> c_int {
	let mut ret = 0;
    for d in pm.devices().unwrap() {
		if d.name().contains("XONE") {
			ret = d.id();
		}
    }
	ret
}

const TIMEOUT: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct MyMidiMessage {
	pub channel: u8,
	pub intensity: u8,
}
impl MyMidiMessage {
	pub fn new(m: pm::types::MidiEvent) -> Self {
		Self {
			channel: m.message.data1,
			intensity: m.message.data2,
		}
	}
}

fn handle_midi_msg(m: MyMidiMessage) -> () {
	println!("{:?}", m);
}



fn main() {

	if std::env::args().skip(1).any(|a| a == "help") {
		println!("[HELP]: type 'cargo run list' to see all available midi devices on this computer");
		println!("[HELP]: type 'cargo run ableton' to use the midi output from the wine server running ableton");
		std::process::exit(0);
	}

	if std::env::args().skip(1).any(|a| a == "list") {
		let pm_ctx = pm::PortMidi::new().unwrap();
		let devices = pm_ctx.devices().unwrap();
		devices.iter().for_each(|d| println!("[MAIN]: device {} {:?} {:?}", d.id(), d.name(), d.direction()));
		std::process::exit(0);
	}
	
	let pm_context = pm::PortMidi::new().unwrap();
	let dev_id = if std::env::args().skip(1).any(|a| a == "Through") { 
		get_midithroughport_id(&pm_context)  
	} else if std::env::args().skip(1).any(|a| a == "wine") {
		get_wine_alsa_input_id(&pm_context)
	} else { get_xonek2_id(&pm_context) };
	// get the device info for the given id
	let info = pm_context.device(dev_id).unwrap();

	println!("Listening on: [{}] => [{}]", info.id(), info.name());
	println!("found dev_id [{}]", dev_id);

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

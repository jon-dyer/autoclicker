#![feature(test)]
pub mod args;
mod click_toggle;
pub mod device;

use std::{
    io::{stdout, IsTerminal},
    path::PathBuf,
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use crate::device::Device;
use click_toggle::ClickToggle;
use input_linux::{sys::input_event, Key, KeyState};
const ANSI_BEEP: &str = "\x07";
const OPEN_ESCAPE: &str = "\x1b[0K";
const CLOSE_ESCAPE: &str = "\x1b[1F";

enum Click {
    Left,
    Right,
}

pub struct State {
    input: Arc<Device>,
    output: Arc<Device>,

    left_bind: u16,
    right_bind: u16,

    cooldown: Duration,
    release_cooldown: Duration,
    find_keycodes: bool,

    beep: bool,
}

pub fn mk_device(chosen_device: Option<String>) -> Device {
    if let Some(name) = chosen_device {
        if let Some(device) = Device::find_device(&name) {
            device
        } else {
            eprintln!("Cannot find device: {name}");
            std::process::exit(1);
        }
    } else {
        Device::select_device()
    }
}

pub fn grab_input(input: Device, grab: bool, grab_kbd: bool) -> GrabbedInput {
    let output = Device::uinput_open(PathBuf::from("/dev/uinput"), "TheClicker").unwrap();
    output.add_mouse_attributes();

    if grab {
        if input.ty.is_keyboard() && !grab_kbd {
            eprintln!("Grab mode is disabled for keyboard!");
            eprintln!("You can use --grab-kbd to override that");
        } else {
            output.copy_attributes(&input);
            input.grab(true).expect("Cannot grab the input device!");
        }
    }

    output.create();
    GrabbedInput(input, output)
}

pub struct GrabbedInput(pub Device, pub Device);

impl State {
    pub fn new(
        cooldown: u64,
        cooldown_press_release: u64,
        left_bind: u16,
        right_bind: u16,
        find_keycodes: bool,
        beep: bool,
        GrabbedInput(input, output): GrabbedInput,
    ) -> Self {
        Self {
            input: Arc::new(input),
            output: Arc::new(output),

            left_bind,
            right_bind,

            cooldown: Duration::from_millis(cooldown),
            find_keycodes,
            beep,
            release_cooldown: Duration::from_millis(cooldown_press_release),
        }
    }

    pub fn start(self) {
        let (tx, rx): (mpsc::Sender<Click>, mpsc::Receiver<Click>) = mpsc::channel();

        let mut events: [input_event; 1] = unsafe { std::mem::zeroed() };
        let input: Arc<Device> = self.input.clone();
        let output: Arc<Device> = self.output.clone();

        let find_keycodes = self.find_keycodes;

        if let Ok(key) = Key::from_code(self.left_bind) {
            println!("Left bind code: {}, key: {key:?}", self.left_bind);
        } else {
            println!("Left bind code: {}", self.left_bind);
        }

        if let Ok(key) = Key::from_code(self.right_bind) {
            println!("Right bind code: {}, key: {key:?}", self.right_bind);
        } else {
            println!("Right bind code: {}", self.right_bind);
        }

        let mut states = ClickToggle::default();

        thread::spawn(move || loop {
            input.read(&mut events).unwrap();

            for event in events.iter() {
                log::debug!("Event: {:?}", event);

                if find_keycodes && event.value == 1 {
                    if let Ok(key) = Key::from_code(event.code) {
                        println!("Keycode: {}, key: {key:?}", event.code);
                    } else {
                        println!("Keycode: {}", event.code);
                    }
                }

                let mut used = false;
                let pressed = event.value == 1;
                if event.code == self.left_bind {
                    if pressed && states.not_left() && !find_keycodes {
                        tx.send(Click::Left).unwrap();
                    }
                    states = states.set_left(pressed);
                    used = true;
                }

                if event.code == self.right_bind {
                    if pressed && states.not_right() && !find_keycodes {
                        tx.send(Click::Right).unwrap();
                    }
                    states = states.set_right(pressed);
                    used = true;
                }

                if !used {
                    output
                        .write(&events)
                        .expect("Cannot write to virtual device!");
                }
            }
        });

        self.event_receiver(rx);
    }

    fn event_receiver(self, receiver: mpsc::Receiver<Click>) {
        let mut auto_clicking: ClickToggle = ClickToggle::default();
        println!();
        print_active(&auto_clicking);
        loop {
            if let Some(received_click) = match &auto_clicking {
                ClickToggle::Neither => receiver.recv().ok(),
                ClickToggle::Left | ClickToggle::Right | ClickToggle::Both => {
                    receiver.try_recv().ok()
                }
            } {
                auto_clicking = match received_click {
                    Click::Left => auto_clicking.toggle_left(),
                    Click::Right => auto_clicking.toggle_right(),
                };

                if self.beep {
                    print!("{}", ANSI_BEEP);
                }

                print_active(&auto_clicking);
            }

            match auto_clicking {
                ClickToggle::Left => self.output.send_key(Key::ButtonLeft, KeyState::PRESSED),
                ClickToggle::Right => self.output.send_key(Key::ButtonRight, KeyState::PRESSED),
                ClickToggle::Both => {
                    self.output.send_key(Key::ButtonLeft, KeyState::PRESSED);
                    self.output.send_key(Key::ButtonRight, KeyState::PRESSED);
                }
                ClickToggle::Neither => (),
            }

            match self.release_cooldown {
                Duration::ZERO => (),
                release_cooldown => thread::sleep(release_cooldown),
            }

            match auto_clicking {
                ClickToggle::Left => self.output.send_key(Key::ButtonLeft, KeyState::RELEASED),
                ClickToggle::Right => self.output.send_key(Key::ButtonRight, KeyState::RELEASED),
                ClickToggle::Both => {
                    self.output.send_key(Key::ButtonLeft, KeyState::RELEASED);
                    self.output.send_key(Key::ButtonRight, KeyState::RELEASED);
                }
                ClickToggle::Neither => (),
            }

            thread::sleep(self.cooldown);
        }
    }
}

fn print_active(toggle: &ClickToggle) {
    reprint(&format!("Active: {}\n", toggle));
}

#[inline]
fn reprint(text: &str) {
    if stdout().is_terminal() {
        print!("{}{}{}", OPEN_ESCAPE, text, CLOSE_ESCAPE);
    } else {
        print!("{}", text);
    }
}

#[cfg(test)]
mod tests {
    use gag::BufferRedirect;
    use std::io::Read;

    use super::*;

    #[test]
    fn print_active_whole_thing() {
        let mut buf = BufferRedirect::stdout().unwrap();

        print_active(&ClickToggle::Left);

        let mut output: String = String::new();
        buf.read_to_string(&mut output).unwrap();

        assert_eq!(&output[..], "Active: left\n");
    }
}

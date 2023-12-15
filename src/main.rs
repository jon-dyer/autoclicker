pub mod args;


use std::{
    fs::{self, File},
    io::Read,
};

use crate::args::Args;
use clap::Parser;
use theclicker::{State, device::DeviceType};

fn main() {
    env_logger::init();
    let Args {
        clear_cache,
        mut cooldown,
        mut cooldown_press_release,
        mut left_bind,
        mut right_bind,
        mut find_keycodes,
        mut no_beep,
        mut no_grab,
        mut use_device,
        mut grab_kbd,
    } = Args::parse();

    if clear_cache {
        let _ = fs::remove_file("/tmp/TheClicker");
    }

    if let Ok(mut file) = File::open("/tmp/TheClicker") {
        println!("Loaded from cache!");
        eprintln!("Args are disabled if we have cache!");
        eprintln!("You can use --clear-cache");
        let mut string = String::default();
        file.read_to_string(&mut string).unwrap();
        Args {
            clear_cache: _,
            cooldown,
            cooldown_press_release,
            left_bind,
            right_bind,
            find_keycodes,
            no_beep,
            no_grab,
            grab_kbd,
            use_device,
        } = ron::from_str::<Args>(&string).unwrap();
    }

    let beep = !no_beep;
    let device = theclicker::mk_device(use_device);

    let input: theclicker::GrabbedInput = theclicker::grab_input(device, !no_grab, grab_kbd);

    let left_bind: u16 = left_bind.unwrap_or(match input.0.ty {
        DeviceType::Mouse => 275,
        DeviceType::Keyboard => 26,
    });

    let right_bind: u16 = right_bind.unwrap_or(match input.0.ty {
        DeviceType::Mouse => 276,
        DeviceType::Keyboard => 27,
    });

    println!("Using: {}", input.0.name);

    let state = State::new(
        cooldown,
        cooldown_press_release,
        left_bind,
        right_bind,
        find_keycodes,
        beep,
        input,
    );

    println!();
    println!("Cooldown is {}ms!", cooldown);
    println!(
        "Cooldown between press and release is {}ms!",
        cooldown_press_release
    );

    state.start();
}

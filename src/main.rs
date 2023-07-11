use std::io::{stdin, Read};
use std::process;
use pit::Collision;
use raw_tty::GuardMode;
use rand::thread_rng;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod snak;
mod backend;
mod pit;

fn main() {

    let (ltx, lrx) = mpsc::channel();
    let (tx, rx) = mpsc::channel();
    
    let mut rng = thread_rng();
    let size = backend::get_size().unwrap();

    let mut stdin = stdin().guard_mode().unwrap();
    stdin.set_raw_mode().unwrap();

    let mut pit = pit::Pit::new(size);
    pit.empty();
    pit.st_screen();
    let mut snak = snak::Snak::new(pit.get_size());

    let mut ticks: u16 = 0;
    let mut score: u16 = 0;
    let mut lock: bool = true;
    let mut speed: u8 = 120;
    
    // auxiliary thread here -> tracking user inputs
    thread::spawn(move || {
        for i in stdin.bytes() {
            // kill switch
            if i.as_ref().is_ok_and(|i| i == &3) {
                println!("\r\x1b[?25h");
                break;
            }
            // start game (small e)
            if i.as_ref().is_ok_and(|i| i == &101) { ltx.send(true).unwrap(); }

            // arrows or capital A, B, C, D 
            let dir = match i.unwrap() {
                65 => Some(snak::Dir::UP),
                66 => Some(snak::Dir::DOWN),
                67 => Some(snak::Dir::RIGHT),
                68 => Some(snak::Dir::LEFT),
                _ => None,
            };
            // send change in direction to main thread
            if let Some(x) = dir { tx.send(x).unwrap(); }
        };
        // will kill the game
        process::exit(0x0);
    }); 
    
    // main thread here -> moves snake around
    loop {
        if lrx.try_recv().is_ok() {
            lock = false;
            pit.empty();
            pit.next_munch(&mut rng);
        };
        if lock { continue; };

        if let Ok(x) = rx.try_recv() { snak.change_dir(x) };

        let pos = snak.munch();
        match pit.is_collision(pos) {
            Collision::No => pit.set(pos, Some(snak.tailoff())),
            Collision::Munch => {
                pit.set(pos, None);
                pit.next_munch(&mut rng);
                score += 1;
                speed = speed.checked_sub(2).unwrap_or(0);
            },
            Collision::Ded => {
                pit.kill_snak(snak.ded());
                break;
            },
        };

        ticks += 1;
        pit.set_status(score, ticks);
        pit.blit();
        thread::sleep(Duration::from_millis(80 + speed as u64))
    }

    loop {};

}

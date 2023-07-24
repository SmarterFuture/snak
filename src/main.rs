use pit::Collision;
use rand::thread_rng;
use raw_tty::GuardMode;
use std::io::{stdin, Read};
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod backend;
mod brainz;
mod pit;
mod snak;

fn main() {
    // game lock channel
    let (ltx, lrx) = mpsc::channel();
    // movement channel
    let (tx, rx) = mpsc::channel();

    let mut rng = thread_rng();
    let size = backend::get_size().unwrap();

    let mut stdin = stdin().guard_mode().unwrap();
    stdin.set_raw_mode().unwrap();
    println!("{:?}", size);
    let mut pit = pit::Pit::new(size);
    pit.empty();
    pit.st_screen();
    let mut snak = snak::Snak::new(pit.get_size());

    let mut ticks: u16 = 0;
    let mut score: u16 = 0;
    let mut lock: bool = true;
    let mut speed: u8 = 120;

    let mut munch: usize = pit.next_munch(&mut rng);

    // auxiliary thread here -> tracking user inputs
    thread::spawn(move || {
        for i in stdin.bytes() {
            // kill switch
            if i.as_ref().is_ok_and(|i| i == &3) {
                println!("\r\x1b[?25h");
                break;
            }
            // start game (small e)
            if i.as_ref().is_ok_and(|i| i == &101) {
                ltx.send(true).unwrap();
            }

            // arrows or capital A, B, C, D
            let dir = match i.unwrap() {
                65 => Some(snak::Dir::UP),
                66 => Some(snak::Dir::DOWN),
                67 => Some(snak::Dir::RIGHT),
                68 => Some(snak::Dir::LEFT),
                _ => None,
            };
            // send change in direction to main thread
            if let Some(x) = dir {
                tx.send(x).unwrap();
            }
        }
        // will kill the game
        process::exit(0x0);
    });

    // main thread here -> moves snake around
    loop {
        if lrx.try_recv().is_ok() {
            lock = false;
            pit.empty();
            munch = pit.next_munch(&mut rng);
        };
        if lock {
            continue;
        };
        if let Ok(x) = rx.try_recv() {
            snak.change_dir(x)
        };

        let pos = snak.munch();
        match pit.is_collision(pos) {
            Collision::No => pit.set(pos, Some(snak.tailoff())),
            Collision::Munch => {
                pit.set(pos, None);
                munch = pit.next_munch(&mut rng);
                score += 1;
                speed = speed.checked_sub(2).unwrap_or(0);
            }
            Collision::Ded => {
                pit.kill_snak(snak.ded());
                break;
            }
        };
        pit.clear();
        let path = brainz::safe_path(&mut pit, &snak, munch).unwrap_or(Vec::new());
        pit.set_path(&path);

        let next = *path.iter().nth(1).unwrap_or(&pos);
        match next {
            x if x + 2 == pos => snak.change_dir(snak::Dir::LEFT),
            x if x - 2 == pos => snak.change_dir(snak::Dir::RIGHT),
            x if x + 2 < pos => snak.change_dir(snak::Dir::UP),
            x if x - 2 > pos => snak.change_dir(snak::Dir::DOWN),
            _ => {}
        };

        ticks += 1;
        pit.set_status(score, ticks);
        pit.blit();
        thread::sleep(Duration::from_millis(20)); //80 + speed as u64))
    }

    loop {}
}

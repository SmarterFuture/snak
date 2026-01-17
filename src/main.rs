use std::{
    env,
    io::{self, Read},
    sync::mpsc,
    thread,
    time::Duration,
};

use rand::thread_rng;
use raw_tty::IntoRawMode;

use crate::{
    backend::{Quad, RenderTarget, Size},
    term_display::{TermScreen, TermStatusLine},
};

mod backend;
mod snake;
mod term_display;

fn play_snake(w: usize, h: usize) -> Result<(), io::Error> {
    let (lock_tx, lock_rx) = mpsc::channel::<bool>();
    let (stop_tx, stop_rx) = mpsc::channel::<bool>();
    let (dir_tx, dir_rx) = mpsc::channel::<snake::Dir>();

    let rng = thread_rng();

    let mut snake = snake::Snake::new(Size::new(w, h), rng);
    let mut screen = TermScreen::new(w, h);
    let mut statusline = TermStatusLine::new(w);
    screen.init()?;
    statusline.init()?;

    let diff_rate = [150, 145, 140, 135, 130];
    let mut sleep;

    // auxiliary thread here -> tracking user inputs
    let input = thread::spawn(move || {
        let mut raw_stdin = io::stdin().into_raw_mode().unwrap();
        let mut buf = [0u8; 1];

        loop {
            raw_stdin.read_exact(&mut buf).unwrap();

            let dir = match buf[0] {
                // kill switch
                3 | b'q' => {
                    stop_tx.send(true).unwrap();
                    break;
                }
                // start game (small e)
                b'e' => {
                    lock_tx.send(true).unwrap();
                    None
                }
                // arrows or capital A, B, C, D
                b'A' => Some(snake::Dir::Up),
                b'B' => Some(snake::Dir::Down),
                b'C' => Some(snake::Dir::Right),
                b'D' => Some(snake::Dir::Left),
                _ => None,
            };

            // send change in direction to main thread
            if let Some(x) = dir {
                dir_tx.send(x).unwrap();
            }
        }
    });

    let mid_x = w / 2;
    let mid_y = h / 2;
    screen.render_text(mid_x, mid_y, "Snake!".to_string(), Quad::Center)?;
    screen.render_text(
        mid_x,
        mid_y + 1,
        "To start, press e!".to_string(),
        Quad::Center,
    )?;

    // waiting for unlock
    if lock_rx.recv().is_ok() {
        snake.start();
    }

    // main thread here -> moves snake around
    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        if let Ok(x) = dir_rx.try_recv() {
            snake.change_dir(x);
        }

        let r_diff = snake.tick_move();
        snake.draw_snake_to(&mut screen)?;
        snake.draw_status_to(&mut statusline)?;

        if let Ok(diff) = r_diff {
            sleep = *diff_rate.get(diff).unwrap_or(&60);
        } else {
            break;
        }

        thread::sleep(Duration::from_millis(sleep))
    }

    screen.render_text(mid_x, mid_y, "You have died!".to_string(), Quad::Center)?;
    screen.render_text(
        mid_x,
        mid_y + 1,
        "To exit press q!".to_string(),
        Quad::Center,
    )?;

    input.join().unwrap();

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut w = 20;
    let mut h = 20;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-w" | "--width" => {
                if let Some(val) = iter.next() {
                    w = val.parse().unwrap();
                    if !(0..=100).contains(&w) {
                        eprintln!("Width is not in range [0, 100]!");
                        return;
                    }
                }
            }
            "-h" | "--height" => {
                if let Some(val) = iter.next() {
                    h = val.parse().unwrap();
                    if !(0..=100).contains(&h) {
                        eprintln!("Height is not in range [0, 100]!");
                        return;
                    }
                }
            }
            _ => {}
        }
    }

    play_snake(w, h).unwrap();
}

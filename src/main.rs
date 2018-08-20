extern crate rand;
extern crate png;

use std::io;
use std::env;
use std::thread;
use std::sync::mpsc;
use std::path::Path;
use std::fs::{self, File};
use png::HasParameters;

const WIDTH:    usize = 1920;
const HEIGHT:   usize = 1080;
const DEFAULT_ITER_MAX: usize = 100;

fn main() {
    // TODO -> .do these obey copy rules? We can assign one to the other and
    // then keep on writing to the first. Doesn't obey move semantics...
    let mut board      = [0u8; WIDTH*HEIGHT];
    let mut next_board = [0u8; WIDTH*HEIGHT];

    let (tx, rx) = mpsc::channel();

    let iter_max: usize = env::args().nth(1)
                                     .unwrap_or(format!("{}", DEFAULT_ITER_MAX))
                                     .parse()
                                     .unwrap();

    // initialize and save first board
    randomize(&mut board);

    png_write(0, board.to_vec()).unwrap();

    let writer_thread = thread::spawn(move || {
        while let Ok(Some((iteration, data))) = rx.recv() {
            png_write(iteration, data).unwrap();
        }
    });

    let start = std::time::Instant::now();
    
    // main loop
    for iteration in 1..=iter_max {
        println!("{}/{}", iteration, iter_max);

        // precursor to multithreading support here
        for (i, _) in board.iter().enumerate() {
            let (row, col) = (i / WIDTH, i % WIDTH);
            game_of_life(&board, &mut next_board, row, col);
        }

        // png_write(iteration, next_board.to_vec()).unwrap();
        tx.send(Some((iteration, next_board.to_vec())))
           .expect("Failed to send on pipe!");

        // I guess there's an implicit copy here since we keep two references
        // and can read from one while writing to the other. In order to keep Rust's
        // rules intact, they can't point to the same memory 
        // unsafe {std::ptr::swap(&mut board, &mut next_board)};
        board = next_board;
    }

    let end = std::time::Instant::now();
    println!("--Completed in {:?}--", end-start);

    // kill the thread and clean up
    tx.send(None).expect("Failed to send on pipe!");
    writer_thread.join().expect("Thread panicked!");
}

// core game logic
fn game_of_life(board: &[u8], next_board: &mut [u8], r: usize, c: usize) {
    // there's an overflow chance here, but as long as the gameboard only holds ones or zeros
    // it shouldn't be an issue
    let count_neighbors: u8 = [
        get_offset(&board, r, c,  0,  1),
        get_offset(&board, r, c,  1,  1),
        get_offset(&board, r, c,  1,  0),
        get_offset(&board, r, c,  1, -1),
        get_offset(&board, r, c,  0, -1),
        get_offset(&board, r, c, -1, -1),
        get_offset(&board, r, c, -1,  0),
        get_offset(&board, r, c, -1,  1),
    ].iter().sum();

    // set the cell at offset (r, c) to a value based on count_neighbors
    if get_offset(&board, r, c, 0, 0) != 0 { // live cell
        if count_neighbors == 2 ||  count_neighbors == 3 {
            next_board[r*WIDTH + c] = 1
        } else {
            next_board[r*WIDTH + c] = 0
        }
    } else { // dead cell
        if count_neighbors == 3 {
            next_board[r*WIDTH + c] = 1
        } else {
            next_board[r*WIDTH + c] = 0
        }
    }
}

fn get_offset(board: &[u8], r: usize, c: usize, dc: i32, dr: i32) -> u8 {
    let row = r as i32 + dr;
    let col = c as i32 + dc;

    // bounds check
    if row < 0 || row >= HEIGHT as i32 || col < 0 || col >= WIDTH as i32 {
        0
    } else {
        board[row as usize * WIDTH + col as usize]
    }
}

fn randomize(board: &mut [u8]) {
    for mut val in board.iter_mut() {
        *val = if rand::random::<u8>() % 2 == 0 {
            1
        } else {
            0
        }
    }
}

// all the image writing. Most of this code is taken from the png crate readme
fn png_write(name: usize, data: Vec<u8>) -> io::Result<()> {
    let _ = fs::create_dir("images");
    let pathname = format!("images/{}.png", name);
    let path = Path::new(&pathname);
    let file = File::create(path)?;

    let mut encoder = png::Encoder::new(file, WIDTH as u32, HEIGHT as u32);
    
    encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    
    let mut writer = encoder.write_header()?;

    // 255 = white, 0 = black
    let image_data: Vec<u8> = data.iter().map(|val| if *val != 0 {255} else {0}).collect();

    writer.write_image_data(&image_data)?; // Save

    Ok(())
}

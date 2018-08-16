extern crate rand;
extern crate png;

use std::path::Path;
use std::fs::File;
use std::io;
use std::env;
use png::HasParameters;

use std::thread;
use std::sync::mpsc;

const WIDTH:    usize = 1920;
const HEIGHT:   usize = 1080;
const DEFAULT_ITER_MAX: usize = 1000;

fn main() {
    let iter_max: usize = env::args().nth(1)
                                     .unwrap_or(format!("{}", DEFAULT_ITER_MAX))
                                     .parse()
                                     .unwrap();

    let mut board      = [0u8; WIDTH*HEIGHT];
    let mut next_board = [0u8; WIDTH*HEIGHT];

    randomize(&mut board);

    png_write(0, board.to_vec()).unwrap();

    let (tx, rx) = mpsc::channel();
    let writer_thread = thread::spawn(move || {
        while let Ok(Some((iteration, data))) = rx.recv() {
            png_write(iteration, data).unwrap();
        }
    });

    for iteration in 1..=iter_max {
        println!("{}/{}", iteration, iter_max);

        for r in 0..HEIGHT {
            for c in 0..WIDTH {
                game_of_life(&board, &mut next_board, r, c);
            }
        }

        // png_write(iteration, next_board.to_vec()).unwrap();
        tx.send(Some((iteration, next_board.to_vec()))).unwrap();

        board = next_board;
    }

    tx.send(None).unwrap();
    writer_thread.join().unwrap();
}

fn game_of_life(board: &[u8], next_board: &mut [u8], r: usize, c: usize) {
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

    // println!("{}", count_neighbors);

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

fn png_write(name: usize, data: Vec<u8>) -> io::Result<()> {
    let pathname = format!("images/{}.png", name);
    let path = Path::new(&pathname);
    let file = File::create(path)?;
    // let ref mut w = BufWriter::new(file);
    // let mut encoder = png::Encoder::new(w, WIDTH as u32, HEIGHT as u32);
    let mut encoder = png::Encoder::new(file, WIDTH as u32, HEIGHT as u32);
    encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    let image_data: Vec<u8> = data.iter().map(|val| if *val != 0 {255} else {0}).collect();

    writer.write_image_data(&image_data)?; // Save
    Ok(())
}
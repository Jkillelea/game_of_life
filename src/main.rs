#![allow(warnings)]
extern crate rand;
#[cfg(feature = "opencl")] 
extern crate ocl;
extern crate png;

use std::io;
use std::env;
use std::thread;
use std::sync::mpsc;
use std::path::Path;
use std::fs::{self, File};
use png::HasParameters;

mod cl_impl;

// screen width, height, and default iterations
const WIDTH:            usize = 1920;
const HEIGHT:           usize = 1080;
const DEFAULT_ITER_MAX: usize = 100;

fn main() {
    let mut board = vec![0u8; WIDTH*HEIGHT];
    let (tx, rx) = mpsc::channel();
    let iter_max: usize = env::args().nth(1)
                                     .unwrap_or("None".into())
                                     .parse()
                                     .unwrap_or(DEFAULT_ITER_MAX);

    // thread for saving all the images
    let writer_thread = thread::spawn(move || {
        while let Ok(Some((iteration, data))) = rx.recv() {
            png_write(iteration, data).unwrap();
        }
    });

    // initialize and save first board
    randomize(&mut board);

    png_write(0, board.to_vec()).unwrap();

    #[cfg(feature = "opencl")] 
    let cl_runner = cl_impl::CL::new(include_str!("life.cl")).unwrap();

    // main loop
    let start = std::time::Instant::now();

    for iteration in 1..=iter_max {
        println!("{}/{}", iteration, iter_max);

        let mut next_board = vec![0u8; WIDTH*HEIGHT];

        #[cfg(not(feature = "opencl"))] // default serial implementation
        for (i, _) in board.iter().enumerate() {
            let (row, col) = (i / WIDTH, i % WIDTH);
            game_of_life(&board, &mut next_board, row, col);
        }

        #[cfg(feature = "opencl")] { // OpenCL implementation
            cl_runner.write(&board).unwrap();
            cl_runner.enq_kernel().unwrap();
            cl_runner.read(&mut next_board).unwrap();
        }

        tx.send(Some((iteration, next_board.to_vec())))
           .expect("Failed to send on pipe!");

        board = next_board;
    }

    let end = std::time::Instant::now();
    println!("--Completed in {:?}--", end-start);

    // kill the thread and clean up
    tx.send(None).expect("Failed to send on pipe!");
    writer_thread.join().expect("Thread panicked!");
}

// serial version of core game logic
fn game_of_life(board: &[u8], next_board: &mut [u8], r: usize, c: usize) {
    // there's a u8 overflow chance here, but as long as the gameboard only holds ones or zeros
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

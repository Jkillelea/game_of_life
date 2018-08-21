#![allow(warnings)]
extern crate rand;
extern crate ocl;
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
    let mut board = vec![0u8; WIDTH*HEIGHT];
    let (tx, rx) = mpsc::channel();
    let iter_max: usize = env::args().nth(1)
                                     .unwrap_or(format!("{}", DEFAULT_ITER_MAX))
                                     .parse()
                                     .unwrap();

    // thread for saving all the images
    let writer_thread = thread::spawn(move || {
        while let Ok(Some((iteration, data))) = rx.recv() {
            png_write(iteration, data).unwrap();
        }
    });

    // initialize and save first board
    randomize(&mut board);

    png_write(0, board.to_vec()).unwrap();

    // OpenCL initialization
    let cl_source = include_str!("life.cl");

    let pro_que = ocl::ProQue::builder().src(cl_source).dims((WIDTH, HEIGHT))
                                .build().unwrap();

    let buffer_in = pro_que.create_buffer::<u8>().unwrap();
    let buffer_out = pro_que.create_buffer::<u8>().unwrap();

    let kernel = pro_que.kernel_builder("life").arg(&buffer_in).arg(&buffer_out)
                        .build().unwrap();

    // main loop
    let start = std::time::Instant::now();

    for iteration in 1..=iter_max {
        println!("{}/{}", iteration, iter_max);

        let mut next_board = vec![0u8; WIDTH*HEIGHT];

        buffer_in.write(&board).enq().unwrap();

        unsafe { kernel.enq().unwrap() };

        buffer_out.read(&mut next_board).enq().unwrap();

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

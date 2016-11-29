use fastq::Parser;
use std::io::{stderr, stdin, Read, Write, Result};
use fastq::{Record, thread_reader};
use std::fs::File;
use std::env::args;

use parasailors as align;

extern crate fastq;
extern crate lz4;
extern crate parasailors;

const BUFSIZE: usize = 1 << 22;
const N_THREADS: usize = 2;


fn main() {
    let file: Box<Read + Send> = match args().nth(1).as_ref().map(String::as_ref) {
        None | Some("-") => { Box::new(stdin()) },
        Some(path) => { Box::new(File::open(path).unwrap()) }
    };
    let file = lz4::Decoder::new(file).unwrap();

    let results = thread_reader(BUFSIZE, 2, file, |reader| {
        let parser = Parser::new(reader);
        let results: Result<Vec<u64>> = parser.parallel_each(N_THREADS, |record_sets| {
            let matrix = align::Matrix::new(align::MatrixType::Identity);
            let profile = align::Profile::new(b"ATTAATCCAT", &matrix);

            let mut thread_total: u64 = 0;
            for record_set in record_sets {
                for record in record_set.iter() {
                    let score = align::local_alignment_score(&profile, record.seq(), 11, 1);
                    if score > 7 {
                        thread_total += 1;
                    }
                }
            }
            thread_total
        });
        results
    }).expect("Reader thread paniced");

    match results {
        Err(e) => { write!(stderr(), "Error in fastq file: {}", e).unwrap() },
        Ok(vals) => { println!("total = {}", vals.iter().sum::<u64>()) },
    }
}

use std::{
    cmp::max,
    fmt::{Debug, Display},
    fs::{self, File},
    io::Write,
    sync::{Arc, Mutex},
    time::Instant,
};

use rayon::prelude::*;

use anyhow::Result;
use itertools::Itertools;

#[derive(Clone, Default)]
struct Word {
    bitword: u32,
    bytes: [u8; 5],
}

impl Word {
    fn new(bytes: &[u8]) -> Option<Word> {
        let bytes: [u8; 5] = bytes.try_into().ok()?;
        let mut bitword = 0;
        let mut len = 0;
        for letter in bytes.iter().cloned() {
            debug_assert!(letter >= b'a');
            debug_assert!(letter <= b'z');
            let offset = letter - b'a';
            if bitword & (1 << offset) == 0 {
                bitword |= 1 << offset;
                len += 1
            }
        }
        match len {
            5 => Some(Word { bitword, bytes }),
            _ => None,
        }
    }

    fn transform(mut self, t: [usize; 26]) -> (usize, Self) {
        let mut msl = 0; // most significant letter
        self.bitword = 0;
        for letter in self.bytes.iter().cloned() {
            let offset = t[(letter - b'a') as usize];
            msl = max(msl, offset);
            self.bitword |= 1 << offset;
        }
        (msl, self)
    }
}

impl Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#b} {self}", self.bitword)
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.bytes))
    }
}

fn words_(mut words: Vec<Word>) -> [Vec<Word>; 26] {
    words.par_sort_unstable_by_key(|w| w.bitword);
    words.dedup_by_key(|w| w.bitword);

    let mut freqs = [0; 26];
    for word in &words {
        for b in word.bytes {
            freqs[(b - b'a') as usize] += 1;
        }
    }

    // create transform where least frequent letter is 25, second least 24, ..., most frequent 0
    let transform: [usize; 26] = freqs
        .iter()
        .enumerate()
        .sorted_unstable_by_key(|(_i, f)| *f)
        .rev()
        .map(|(i, _f)| i) // letters now in sorted order from most to least frequent
        .enumerate()
        .sorted_unstable_by_key(|(_i_transformed, i_letter)| *i_letter) // sort again, s.t. the letter corresponds with index in transform
        .map(|(i_transformed, _i_letter)| i_transformed)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let mut indexed_words: [Vec<Word>; 26] = Default::default();
    for word in words {
        let (msl, word) = word.transform(transform);
        indexed_words[msl].push(word);
    }
    indexed_words
}

fn next_free_letter(filter: u32) -> Option<usize> {
    (0..26).rev().filter(|n| filter & (1 << n) == 0).next()
}

fn solve14(
    words: &[Vec<Word>; 26],
    skipped: bool,
    filter: u32,
    output: &Arc<Mutex<File>>,
    solution: &mut [Word; 5],
    i: usize,
) {
    let letter = next_free_letter(filter).unwrap();
    match i {
        4 => {
            for word in words[letter].iter() {
                if word.bitword & filter == 0 {
                    let mut file = output.lock().unwrap();
                    writeln!(
                        file,
                        "{} {} {} {} {word}",
                        solution[0], solution[1], solution[2], solution[3]
                    )
                    .unwrap();
                }
            }
        }
        _ => {
            for word in words[letter].iter() {
                if word.bitword & filter == 0 {
                    solution[i] = word.clone();
                    solve14(
                        words,
                        skipped,
                        filter | word.bitword,
                        output,
                        solution,
                        i + 1,
                    );
                }
            }
        }
    }
    if !skipped {
        solve14(words, true, filter | 1 << letter, output, solution, i);
    }
}

fn main() -> Result<()> {
    let start = Instant::now();
    let words = load_words("words_alpha.txt")?;
    let loaded = Instant::now();

    let transformed_words = words_(words);
    let transformed = Instant::now();

    let output = File::create("solution.txt")?;
    solve(&transformed_words, output);

    println!(
        "loading words     {} us",
        loaded.duration_since(start).as_micros()
    );
    println!(
        "transformed words {} us",
        transformed.duration_since(loaded).as_micros()
    );
    println!("solved            {} ms", transformed.elapsed().as_millis());
    println!("total             {} ms", start.elapsed().as_millis());
    Ok(())
}

fn load_words(filepath: &str) -> Result<Vec<Word>> {
    Ok(fs::read(filepath)?
        .par_split(|b| *b == b'\n')
        .filter_map(|l| Word::new(l))
        .collect())
}

fn solve(words: &[Vec<Word>; 26], output_file: File) {
    let output_mtx = Arc::new(Mutex::new(output_file));

    words[25].par_iter().for_each(|word| {
        let mut solution: [Word; 5] = Default::default();
        solution[0] = word.clone();
        let output = Arc::clone(&output_mtx);
        solve14(words, false, word.bitword, &output, &mut solution, 1);
    });

    words[24].par_iter().for_each(|word| {
        let mut solution: [Word; 5] = Default::default();
        solution[0] = word.clone();
        let output = Arc::clone(&output_mtx);
        let filter = word.bitword | 1 << 25;
        solve14(words, true, filter, &output, &mut solution, 1);
    });
}

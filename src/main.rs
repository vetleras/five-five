use std::{
    cmp::max,
    fmt::{Debug, Display},
    fs::{self, File},
    io::{Result, Write},
    sync::Mutex,
    time::Instant,
};

use itertools::Itertools;
use rayon::prelude::*;

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

fn next_free_letter(filter: u32) -> Option<usize> {
    (0..26).rev().filter(|n| filter & (1 << n) == 0).next()
}

struct Solver {
    words: [Vec<Word>; 26],
    output: Mutex<File>,
}

impl Solver {
    fn new(mut words: Vec<Word>, output: File) -> Self {
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
            .into_iter()
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

        Solver {
            words: indexed_words,
            output: Mutex::new(output),
        }
    }

    fn solve(&self) {
        self.words[25].par_iter().for_each(|word| {
            let mut solution: [Word; 5] = Default::default();
            solution[0] = word.clone();
            self.solve14(word.bitword, false, &mut solution, 1);
        });

        self.words[24].par_iter().for_each(|word| {
            let mut solution: [Word; 5] = Default::default();
            solution[0] = word.clone();
            self.solve14(word.bitword | 1 << 25, true, &mut solution, 1);
        });
    }

    fn solve14(&self, filter: u32, skipped: bool, solution: &mut [Word; 5], i: usize) {
        let letter = next_free_letter(filter).unwrap();
        match i {
            4 => {
                for word in &self.words[letter] {
                    if word.bitword & filter == 0 {
                        let mut file = self.output.lock().unwrap();
                        let s = &solution;
                        writeln!(file, "{} {} {} {} {word}", s[0], s[1], s[2], s[3]).unwrap();
                    }
                }
            }
            _ => {
                for word in &self.words[letter] {
                    if word.bitword & filter == 0 {
                        solution[i] = word.clone();
                        self.solve14(filter | word.bitword, skipped, solution, i + 1);
                    }
                }
            }
        }
        if !skipped {
            self.solve14(filter | 1 << letter, true, solution, i);
        }
    }
}

fn main() -> Result<()> {
    let start = Instant::now();
    let words = fs::read("words_alpha.txt")?
        .par_split(|b| *b == b'\n')
        .filter_map(|l| Word::new(l))
        .collect();

    let output = File::create("solutions.txt")?;
    let solver = Solver::new(words, output);
    solver.solve();

    println!("{} us", start.elapsed().as_micros());
    Ok(())
}

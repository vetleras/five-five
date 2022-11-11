use std::{
    cmp::max,
    fmt::{Debug, Display},
    fs::{self, File},
    io::Write,
    time::Instant,
};

use anyhow::{bail, Result};
use itertools::Itertools;

#[derive(Eq, PartialOrd, Ord, Clone, Default)]
struct Word {
    bitword: u32,
    bytes: [u8; 5],
}

impl Word {
    fn new(bytes: &[u8]) -> Result<Word> {
        let bytes: [u8; 5] = bytes.try_into()?;
        let mut bitword = 0;
        let mut len = 0;
        for letter in bytes.iter().cloned() {
            if letter < b'a' && letter > b'z' {
                bail!("invalid letter {letter}")
            }
            let offset = letter - b'a';
            if bitword & (1 << offset) == 0 {
                bitword |= 1 << offset;
                len += 1
            }
        }
        match len {
            5 => Ok(Word { bitword, bytes }),
            _ => bail!("invalid bitword length {len}"),
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

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.bitword == other.bitword
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

fn words(bytes: &[u8]) -> [Vec<Word>; 26] {
    let words: Vec<Word> = bytes
        .split(|b| *b == b'\n') // split on \n
        .map(|s| match s.last() {
            Some(b'\r') => &s[0..s.len() - 1], // strip \r
            _ => s,
        })
        .filter_map(|line| Word::new(line).ok())
        .sorted_unstable() // sort for dedup
        .dedup() // remove anagrams
        .collect();

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

fn solve(words: &[Vec<Word>; 26], filter: u32, skipped: bool, i: usize) -> Vec<[Word; 5]> {
    let mut solutions = Vec::new();
    match i {
        0 => {
            for word in words[25].iter() {
                for mut solution in solve(words, word.bitword, false, 1) {
                    solution[0] = word.clone();
                    solutions.push(solution);
                }
            }
            for word in words[24].iter() {
                for mut solution in solve(words, word.bitword | 1 << 25, true, 1) {
                    solution[0] = word.clone();
                    solutions.push(solution);
                }
            }
        }

        4 => {
            let letter = next_free_letter(filter).unwrap();
            for word in words[letter].iter() {
                if word.bitword & filter == 0 {
                    let mut solution: [Word; 5] = Default::default();
                    solution[4] = word.clone();
                    solutions.push(solution);
                }
            }
            if !skipped {
                solutions.append(&mut solve(words, filter | 1 << letter, true, 4));
            }
        }

        _ => {
            let letter = next_free_letter(filter).unwrap();
            for word in words[letter].iter() {
                if word.bitword & filter == 0 {
                    for mut solution in solve(words, filter | word.bitword, false, i + 1) {
                        solution[i] = word.clone();
                        solutions.push(solution);
                    }
                }
            }
            if !skipped {
                solutions.append(&mut solve(words, filter | 1 << letter, true, i));
            }
        }
    };
    solutions
}

fn main() -> Result<()> {
    let start = Instant::now();

    let bytes = fs::read("words_alpha.txt")?;
    let words = words(&bytes);
    let solutions = solve(&words, 0, false, 0);

    let mut output = File::create("solution.txt")?;
    for s in &solutions {
        writeln!(output, "{} {} {} {} {}", s[0], s[1], s[2], s[3], s[4])?;
    }
    println!("{} solutions", solutions.len());
    println!("{:.2} seconds", start.elapsed().as_secs_f32());
    Ok(())
}

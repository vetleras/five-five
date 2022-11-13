use std::{
    cmp::max,
    fmt::{Debug, Display},
    fs::{self, File},
    io::Write,
    time::Instant,
};

use anyhow::{bail, Result};
use itertools::Itertools;

#[derive(Clone, Default)]
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
        .sorted_unstable_by_key(|w| w.bitword) // sort for dedup
        .dedup_by(|w1, w2| w1.bitword == w2.bitword) // remove anagrams
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

fn solve(words: &[Vec<Word>; 26], output: &mut File) {
    let mut solution = Vec::with_capacity(5);
    for word in words[25].iter() {
        solution.push(word.clone());
        solve14(words, false, word.bitword, output, &mut solution);
        solution.pop();
    }
    for word in words[24].iter() {
        solution.push(word.clone());
        solve14(words, true, word.bitword | 1 << 25, output, &mut solution);
        solution.pop();
    }
}

fn solve14(
    words: &[Vec<Word>; 26],
    skipped: bool,
    filter: u32,
    output: &mut File,
    solution: &mut Vec<Word>,
) {
    let letter = next_free_letter(filter).unwrap();
    match solution.len() {
        4 => {
            for word in words[letter].iter() {
                if word.bitword & filter == 0 {
                    writeln!(output, "{} {} {} {} {word}", solution[0], solution[1], solution[2], solution[3]).unwrap();
                }
            }
        }
        _ => {
            for word in words[letter].iter() {
                if word.bitword & filter == 0 {
                    solution.push(word.clone());
                    solve14(words, skipped, filter | word.bitword, output, solution);
                    solution.pop();
                }
            }
        }
    }
    if !skipped {
        solve14(words, true, filter | 1 << letter, output, solution);
    }
}

fn main() -> Result<()> {
    let start = Instant::now();
    let bytes = fs::read("words_alpha.txt")?;
    let words = words(&bytes);

    let mut output = File::create("solution.txt")?;

    let start_search = Instant::now();
    solve(&words, &mut output);

    println!("total  {} ms", start.elapsed().as_millis());
    println!("search {} ms", start_search.elapsed().as_millis());
    Ok(())
}

use std::{
    cmp::max,
    fmt::{Debug, Display},
    fs::{self, File},
    io::Write, time::Instant,
};

use anyhow::{bail, Result};

#[derive(Eq, PartialOrd, Ord, Clone, Default)]
struct Word {
    bitword: u32,
    bytes: [u8; 5],
}

impl Word {
    fn new(bytes: &[u8]) -> Result<(u8, Word)> {
        let bytes: [u8; 5] = bytes.try_into()?;
        let mut bitword = 0;
        let mut len = 0;
        let mut msl = 0; // most significant letter
        for letter in bytes.iter().cloned() {
            if letter < b'a' && letter > b'z' {
                bail!("invalid letter {letter}")
            }
            let offset = letter - b'a';
            msl = max(msl, offset);
            if bitword & (1 << offset) == 0 {
                bitword |= 1 << offset;
                len += 1
            }
        }
        match len {
            5 => Ok((msl, Word { bitword, bytes })),
            _ => bail!("invalid bitword length {len}"),
        }
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
    let mut words: [Vec<Word>; 26] = Default::default();
    let lines = bytes.split(|b| *b == b'\n').map(|s| match s.last() {
        Some(b'\r') => &s[0..s.len() - 1],
        _ => s,
    });
    for line in lines {
        if let Ok((msl, word)) = Word::new(line) {
            words[msl as usize].push(word)
        }
    }
    for word_group in &mut words {
        word_group.sort();
        word_group.dedup();
    }
    words
}

fn next_free_letter(filter: u32) -> Option<usize> {
    (0..26).rev().filter(|n| filter & (1 << n) == 0).next()
}

fn solve(words: &[Vec<Word>; 26]) -> Vec<[Word; 5]> {
    let mut solutions = Vec::new();
    for word in words[25].iter() {
        for mut solution in solve14(words, word.bitword, false, 1) {
            solution[0] = word.clone();
            solutions.push(solution);
        }
    }
    for word in words[24].iter() {
        for mut solution in solve14(words, word.bitword | 1 << 25, true, 1) {
            solution[0] = word.clone();
            solutions.push(solution);
        }
    }
    solutions
}

// the separation of solve and solve14 cuts execution time in half
fn solve14(words: &[Vec<Word>; 26], filter: u32, skipped: bool, i: usize) -> Vec<[Word; 5]> {
    let mut solutions = Vec::new();
    let letter = next_free_letter(filter).unwrap();
    for word in words[letter].iter() {
        if word.bitword & filter == 0 {
            if i == 4 {
                let mut solution: [Word; 5] = Default::default();
                solution[4] = word.clone();
                solutions.push(solution);
            } else {
                for mut solution in solve14(words, filter | word.bitword, false, i + 1) {
                    solution[i] = word.clone();
                    solutions.push(solution);
                }
            }
        }
    }
    if !skipped {
        solutions.append(&mut solve14(words, filter | 1 << letter, true, i));
    }
    solutions
}

fn main() -> Result<()> {
    let start = Instant::now();

    let bytes = fs::read("words_alpha.txt")?;
    let words = words(&bytes);
    let solutions = solve(&words);

    let mut output = File::create("solution.txt")?;
    for s in &solutions {
        writeln!(output, "{} {} {} {} {}", s[0], s[1], s[2], s[3], s[4])?;
    }
    println!("{} solutions", solutions.len());
    println!("{:.2} seconds", start.elapsed().as_secs_f32());
    Ok(())
}

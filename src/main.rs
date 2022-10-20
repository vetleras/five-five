use std::{collections::HashSet, fmt, fs::read_to_string};

use anyhow::{bail, Error, Result};

#[derive(PartialEq, Eq, Hash)]
struct Word(u32);

fn bit_to_char(b: u8) -> char {
    (b + 97) as char
}

fn char_to_bit(c: char) -> Result<u8> {
    if c.is_ascii_alphabetic() {
        Ok(c.to_ascii_lowercase() as u8 - 97)
    } else {
        bail!("non-ascii char {c}")
    }
}

impl Word {
    fn len(&self) -> usize {
        let (mut n, mut len) = (self.0, 0);
        while n != 0 {
            if n & 1 == 1 {
                len += 1;
            }
            n >>= 1;
        }
        return len;
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut n: u32 = self.0;
        let mut i: u8 = 0;
        while n != 0 {
            if n & 1 == 1 {
                write!(f, "{}", (bit_to_char(i).to_string()))?
            }
            n >>= 1;
            i += 1;
        }
        Ok(())
    }
}

impl TryFrom<&str> for Word {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut word = 0;
        for c in value.chars() {
            word |= 1 << char_to_bit(c)?;
        }
        Ok(Word(word))
    }
}

fn words_from_file(filepath: &str) -> Result<HashSet<Word>> {
    Ok(read_to_string(filepath)?
        .lines()
        .filter(|x| x.len() == 5)
        .filter_map(|x| Word::try_from(x).ok())
        .filter(|x| x.len() == 5)
        .collect())
}

fn main() -> Result<()> {
    dbg!(words_from_file("words_alpha.txt")?.len());

    Ok(())
}

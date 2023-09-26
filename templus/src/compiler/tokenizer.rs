use std::{error::Error, fmt::Display};

#[derive(Debug)]
enum MarkerType {
    Variable,
    Comment,
    Block,
    Html,
}

#[derive(Debug)]
pub struct Span {
    start: usize,
    end: usize,
    marker_type: MarkerType,
}

// #[derive(Debug)]
// pub struct TokenizerError;
// impl Error for TokenizerError {}



pub fn tokenize(code: &str) -> Result<Vec<Span>,()> {
    let mut blocks = vec![];
    let mut offset = 0;

    loop {
        let (start_type, next_start) = match find_next_start(code.as_bytes(), offset) {
            Some(marker_type) => marker_type,
            None => break,
        };
        offset = next_start + 1;

        let (end_type, next_end) = match find_next_end(code.as_bytes(), offset) {
            Some(marker_type) => marker_type,
            None => break,
        };

        offset = next_end + 1;

        let s = Span {
            start: next_start + 2,
            end: next_end - 2,
            marker_type: start_type,
        };
        println!("{:?}", s);
        blocks.push(s);

        if offset > code.len() {
            break;
        }
    }

    Ok(blocks)
}


fn memchr(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&x| x == needle)
}

fn find_next_start(code: &[u8], offset: usize) -> Option<(MarkerType, usize)> {
    let mut o = 0;
    loop {
        let idx = match memchr(&code[(offset + o)..], b'{') {
            Some(idx) => idx,
            None => return None,
        };


        match code.get(offset + idx + 1) {
            Some(b'{') => return Some((MarkerType::Variable, offset + idx)),
            Some(b'%') => return Some((MarkerType::Block, offset + idx)),
            Some(b'#') => return Some((MarkerType::Comment, offset + idx)),
            _ => match offset + idx + o >= code.len() {
                true => return None,
                false => o += idx + 1,
            }
        }
    }
}

fn find_next_end(code: &[u8], offset: usize) -> Option<(MarkerType, usize)> {
    let mut o = 0;
    loop {
        let idx = match memchr(&code[(offset + o)..], b'}') {
            Some(idx) => idx,
            None => return None,
        };

        match code.get(offset + idx - 1) {
            Some(b'}') => return Some((MarkerType::Variable, offset + idx)),
            Some(b'%') => return Some((MarkerType::Block, offset + idx)),
            Some(b'#') => return Some((MarkerType::Comment, offset + idx)),
            _ => match offset + idx + o >= code.len() {
                true => return None,
                false => o += idx + 1,
            }
        }
    }
}

#[derive(Debug)]
enum BlockType {
    Variable,
    Comment,
    Block,
    Html,
}

#[derive(Debug)]
pub struct BlockSpan {
    start: usize,
    end: usize,
    block_type: BlockType,
}

#[derive(Debug)]
pub enum TokenizerError {
    InvalidSyntax,
    UnclosedBlock,
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TokenizerError {
    fn description(&self) -> &str {
        "Syntax error"
    }
}

pub struct Tokenizer<'a> {
    code: &'a [u8],
    offset: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Tokenizer { code, offset: 0 }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Result<BlockSpan, TokenizerError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.code.len() {
            return None;
        }

        let (start_type, next_start) = match find_next_start(self.code, self.offset) {
            Some((block_type, next_start)) => (block_type, next_start),
            None => {
                return Some(Ok(BlockSpan {
                    start: self.offset,
                    end: self.code.len(),
                    block_type: BlockType::Html,
                }))
            }
        };

        if next_start > self.offset {
            let span = BlockSpan {
                start: self.offset,
                end: self.offset + next_start,
                block_type: BlockType::Html,
            };
            self.offset = next_start;
            return Some(Ok(span));
        }

        let (end_type, next_end) = match find_next_end(self.code, self.offset) {
            Some((end_type, next_end)) => (end_type, next_end),
            None => return Some(Err(TokenizerError::UnclosedBlock)),
        };

        let block = Some(Ok(BlockSpan {
            start: self.offset,
            end: next_end + 1,
            block_type: start_type,
        }));

        self.offset = next_end + 1;

        return block;
    }
}

fn next_block(code: &[u8], offset: usize) -> Option<(BlockType, usize)> {
    match find_next_start(code, offset) {
        Some((block_type, next_start)) => Some((block_type, next_start)),
        None => None,
    }
}

fn find_next_start(code: &[u8], offset: usize) -> Option<(BlockType, usize)> {
    let mut o = 0;
    loop {
        let idx = match memchr(&code[(offset + o)..], b'{') {
            Some(idx) => idx,
            None => return None,
        };

        match code.get(offset + idx + 1) {
            Some(b'{') => return Some((BlockType::Variable, offset + idx)),
            Some(b'%') => return Some((BlockType::Block, offset + idx)),
            Some(b'#') => return Some((BlockType::Comment, offset + idx)),
            _ => match offset + idx + o >= code.len() {
                true => return None,
                false => o += idx + 1,
            },
        }
    }
}

fn find_next_end(code: &[u8], offset: usize) -> Option<(BlockType, usize)> {
    let mut o = 0;
    loop {
        let idx = match memchr(&code[(offset + o)..], b'}') {
            Some(idx) => idx,
            None => return None,
        };

        match code.get(offset + idx - 1) {
            Some(b'}') => return Some((BlockType::Variable, offset + idx)),
            Some(b'%') => return Some((BlockType::Block, offset + idx)),
            Some(b'#') => return Some((BlockType::Comment, offset + idx)),
            _ => match offset + idx + o >= code.len() {
                true => return None,
                false => o += 1,
            },
        }
    }
}

fn memchr(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&x| x == needle)
}

#[cfg(test)]
mod tests {
    use crate::compiler::tokenizer::Tokenizer;

    #[test]
    fn test_tokenizer_basic() {
        let tmpl = "<html>{% block html %}<p>Hello</p>{% end %}{% block js %}<script>alert('{{ foo }}')</script>{% end %}</html>";
        let mut tokenizer = Tokenizer::new(tmpl.as_bytes());
        loop {
            match tokenizer.next() {
                None => break,
                Some(token) => {
                    let t = token.expect("Invalid token");
                    println!("{:?} :: {:?}", t.block_type, &tmpl[t.start..t.end]);
                }
            }
        }
    }
}

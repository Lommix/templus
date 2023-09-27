use super::error::TemplusCompilerError;


#[derive(Debug)]
pub(crate) enum BlockType {
    Variable,
    Comment,
    Code,
    Html,
}

#[derive(Debug)]
pub struct BlockSpan<'a> {
    start: usize,
    end: usize,
    code: &'a [u8],
    block_type: BlockType,
}

impl <'a> BlockSpan<'a> {
    pub(crate) fn code(&self) -> &'a str {
        std::str::from_utf8(self.code).unwrap().trim()
    }
    pub(crate) fn block_type(&self) -> &BlockType {
        &self.block_type
    }
}


/// The Tokenizer
/// takes raw byte sequence and splits it into meta tokens @[BlockType]
pub struct Tokenizer<'a> {
    code: &'a [u8],
    offset: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Tokenizer { code, offset: 0 }
    }
}

impl <'a> Iterator for Tokenizer<'a> {
    type Item = Result<BlockSpan<'a>, TemplusCompilerError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.code.len() {
            return None;
        }

        let (start_type, next_start) = match find_next_start(self.code, self.offset) {
            Some((block_type, next_start)) => (block_type, next_start),
            None => {
                let r = Some(Ok(BlockSpan {
                    code: &self.code[self.offset..],
                    start: self.offset,
                    end: self.code.len(),
                    block_type: BlockType::Html,
                }));
                self.offset = self.code.len();
                return r;
            }
        };

        if next_start > 0 {
            let span = BlockSpan {
                code: &self.code[self.offset..(self.offset + next_start)],
                start: self.offset,
                end: self.offset + next_start,
                block_type: BlockType::Html,
            };
            self.offset += next_start;
            return Some(Ok(span));
        }

        let (end_type, next_end) = match find_next_end(self.code, self.offset) {
            Some((end_type, next_end)) => (end_type, next_end),
            None => {
                return Some(Err(TemplusCompilerError::UnclosedBlock(format!(
                    "{:?}",
                    start_type
                ))))
            }
        };


        let block_start = self.offset;
        let block_end = self.offset + next_end + 1;
        let code = &self.code[block_start + 2 ..block_end - 2];

        let block = Some(Ok(BlockSpan {
            code,
            start: self.offset,
            end: self.offset + next_end + 1,
            block_type: start_type,
        }));

        self.offset += next_end + 1;

        return block;
    }
}

fn skip_whitespace(code: &[u8]) -> Option<usize> {
    let mut local_offset = 0;
    loop {
        if local_offset >= code.len() {
            return None;
        }

        if code[local_offset] == b' ' || code[local_offset] == b'\t' {
            local_offset += 1;
        } else {
            return Some(local_offset);
        }
    }
}

fn next_block(code: &[u8], offset: usize) -> Option<(BlockType, usize)> {
    match find_next_start(code, offset) {
        Some((block_type, next_start)) => Some((block_type, next_start)),
        None => None,
    }
}

fn find_next_start(code: &[u8], offset: usize) -> Option<(BlockType, usize)> {
    let mut local_offset = 0;
    loop {
        let idx = match memchr(&code[(offset + local_offset)..], b'{') {
            Some(idx) => idx,
            None => return None,
        };

        match code.get(offset + idx + local_offset + 1) {
            Some(b'{') => return Some((BlockType::Variable, idx + local_offset)),
            Some(b'%') => return Some((BlockType::Code, idx + local_offset)),
            Some(b'#') => return Some((BlockType::Comment, idx + local_offset)),
            _ => match offset + idx + local_offset >= code.len() {
                true => return None,
                false => local_offset += idx.max(1),
            },
        }
    }
}

fn find_next_end(code: &[u8], offset: usize) -> Option<(BlockType, usize)> {
    let mut local_offset = 0;
    loop {
        let idx = match memchr(&code[(offset + local_offset)..], b'}') {
            Some(idx) => idx,
            None => return None,
        };

        match code.get(offset + idx + local_offset - 1) {
            Some(b'}') => return Some((BlockType::Variable, idx + local_offset)),
            Some(b'%') => return Some((BlockType::Code, idx + local_offset)),
            Some(b'#') => return Some((BlockType::Comment, idx + local_offset)),
            _ => match offset + idx + local_offset >= code.len() {
                true => return None,
                false => local_offset += idx.max(1),
            },
        }
    }
}

fn memchr(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&x| x == needle)
}

#[cfg(test)]
mod tests {
    use super::memchr;
    use crate::compiler::tokenizer::{find_next_end, find_next_start, Tokenizer};

    #[test]
    fn test_memchr() {
        let t = &[1, 2, 3, 4, 5];
        assert_eq!(Some(1), memchr(&t[..], 2));
        assert_eq!(Some(0), memchr(&t[1..], 2));
    }

    #[test]
    fn test_start_end() {
        let bytes = "{3{% block %}3}".as_bytes();
        let (start_type, start_offset) = find_next_start(bytes, 0).unwrap();
        let (end_type, end_offset) = find_next_end(bytes, 0).unwrap();
        assert_eq!(start_offset, 2);
        assert_eq!(end_offset, 12);
    }

    #[test]
    fn test_tokenizer_iterator() {
        // let tmpl = std::fs::read_to_string("test_templates/1.html").unwrap();
        let tmpl = "<html>{% block 'html' %}<p>Hello</p>{% end %}{% block js %}<script>alert('{{ foo }}')</script>{% end %}</html>";
        let mut tokenizer = Tokenizer::new(tmpl.as_bytes());
        let mut timeout = 0;
        loop {
            match tokenizer.next() {
                None => break,
                Some(token) => {
                    let t = token.expect("Invalid token");
                    // println!("{:?} :: {:?}", t.block_type, std::str::from_utf8(t.code).unwrap() );
                }
            }
            timeout += 1;
            if timeout > 100 {
                break;
            }
        }
    }
}

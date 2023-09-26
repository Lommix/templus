#[allow(unused_variables)]
#[allow(dead_code)]

mod compiler;

#[cfg(test)]
mod tests {
    use crate::compiler::{self, tokenizer::tokenize};


    #[test]
    fn it_works() {
        let content = std::fs::read_to_string("test_templates/1.html").unwrap();
        let tokens = tokenize(&content);

        for token in tokens {
            println!("{:?}", token);
        }
    }
}

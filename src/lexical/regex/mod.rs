use std::str::Chars;

#[derive(Debug)]
pub enum RegEx {
    Character(char),
    Concat(Box<RegEx>, Box<RegEx>),
    KleeneClosure(Box<RegEx>),
    PositiveClosure(Box<RegEx>),
    Union(Box<RegEx>, Box<RegEx>)
}

impl RegEx {
    fn parse_primary(chars: &mut Chars) -> Self {
        let char = chars.next().unwrap();
        if char == '(' {
            let result = Self::parse_union(chars);
            chars.next().expect("Malformed RegEx");
            return result;
        }
        RegEx::Character(char)
    }

    fn parse_closure(chars: &mut Chars) -> Self {
        let mut result = Self::parse_primary(chars);
        while let Some(char) = chars.clone().next() {
            match char {
                '*' => {
                    chars.next();
                    result = RegEx::KleeneClosure(Box::new(result));
                }
                '+' => {
                    chars.next();
                    result = RegEx::PositiveClosure(Box::new(result));
                }
                _ => break
            }
        }
        result
    }

    fn parse_concat(chars: &mut Chars) -> Self {
        let mut result = Self::parse_closure(chars);
        if let Some(char) = chars.clone().next() {
            // Everything that is just a normal character
            if char == ')' || char == '|' {
                return result;
            }
            result = RegEx::Concat(Box::new(result), Box::new(Self::parse_concat(chars)));
        }
        result
    }

    fn parse_union(chars: &mut Chars) -> Self {
        let mut result = Self::parse_concat(chars);
        while chars.clone().next() == Some('|') {
            chars.next();
            result = RegEx::Union(Box::new(result), Box::new(Self::parse_union(chars)));
        }
        result
    }

    pub fn new_from_chars(chars: &mut Chars) -> Self {
        if chars.clone().next().is_none() {
            panic!("Malformed RegEx");
        }
        Self::parse_union(chars)
    }
}
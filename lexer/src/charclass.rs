use unicode::{is,UnicodeProperty};

pub fn is_line_terminator(c: char) -> bool {
    match c {
        '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => true,
        _ => false,
    }
}

pub fn is_ws(c: char) -> bool {
    match c {
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' |
        '\u{00A0}' | '\u{1680}' | '\u{2000}' | '\u{2001}' |
        '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' |
        '\u{2006}' | '\u{2007}' | '\u{2008}' | '\u{2009}' |
        '\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{FEFF}' => true,
        _ => false,
    }
}

pub fn is_id_start(c: char) -> bool {
    match c {
        c if is(c, UnicodeProperty::IdStart) => true,
        '\\' => true,
        '$'  => true,
        '_'  => true,
        _    => false,
    }
}

pub fn is_id_continue(c: char) -> bool {
    match c {
        c if is(c, UnicodeProperty::IdContinue) => true,
        '\\'       => true,
        '$'        => true,
        '\u{200C}' => true, // ZWNJ
        '\u{200D}' => true, // ZWJ
        _          => false,
    }
}

pub fn is_escapable_char(c: char) -> bool {
    match c {
        x if x.is_ascii_digit()    => false,
        x if is_line_terminator(x) => false,
        'x' => false,
        'u' => false,
        _   => true,
    }
}

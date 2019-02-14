macro_rules! test_log {
    ($($arg:tt)*) => {};
}

#[macro_export]
macro_rules! bytes_concat {
    ($($byte_strings:expr),*$(,)*) => {{
        &[$($byte_strings),*].concat()
    }}
}

pub fn bytes_lines<'a>(bytes: &'a [u8]) -> impl Iterator<Item = &'a [u8]> {
    bytes.split(|&b| b == b'\n')
}

pub fn reflow(s: &str, width: usize) -> String {
    if width == 0 || s.len() == 0 {
        return String::new();
    }
    let mut output = String::with_capacity(s.len() + s.len() / width);

    let mut x = 0;
    for word in s.split_whitespace() {
        x += word.len();

        if x == width && x == word.len() {
            output.push_str(word);
            continue;
        }

        if x >= width {
            output.push('\n');

            x = word.len();
        } else if x > word.len() {
            output.push(' ');

            x += 1;
        }
        output.push_str(word);
    }

    output
}

pub fn bytes_reflow(bytes: &[u8], width: usize) -> Vec<u8> {
    if width == 0 || bytes.len() == 0 {
        return Vec::new();
    }
    test_log!(width);
    let mut output = Vec::with_capacity(bytes.len() + bytes.len() / width);

    let mut x = 0;
    for word in bytes_split_whitespace(bytes) {
        test_log!(word);
        x += word.len();
        test_log!(x);
        test_log!(output);
        if x == width && x == word.len() {
            output.extend(word.iter());
            continue;
        }

        if x >= width {
            output.push(b'\n');

            x = word.len();
        } else if x > word.len() {
            output.push(b' ');

            x += 1;
        }
        output.extend(word.iter());
    }

    output
}

pub fn bytes_reflow_in_place(bytes: &mut Vec<u8>, width: usize) {
    if width == 0 || bytes.len() == 0 {
        test_log!("width == 0 || bytes.len() == 0");
        return;
    }
    test_log!("start");
    test_log!((bytes.clone(), width));
    let used_len = bytes.len();
    let extra = bytes.len() / width;
    bytes.reserve(extra);

    //fill with 0's to capacity
    for _ in 0..extra {
        bytes.push(0);
    }

    //shift used parts down to the end
    {
        let mut index = bytes.len() - 1;
        test_log!((0..used_len).rev());
        for i in (0..used_len).rev() {
            test_log!(index);
            test_log!(i);
            bytes[index] = bytes[i];
            index -= 1;
        }
    }

    let mut index = 0;
    {
        //full length - used_len == (used_len + extra) - used_len == extra
        let shifted_start = extra;
        let mut next_i = shifted_start;
        test_log!(bytes);
        test_log!(shifted_start);
        //scan from the start of the (moved) used portion and copy it back to the front
        //inserting newlines where appropiate.
        let mut x = 0;
        while let Some((w_i, len)) = bytes_next_word(&bytes, &mut next_i) {
            test_log!((w_i, len));
            test_log!(&bytes[w_i..w_i + len]);
            x += len;
            test_log!(x);

            if x == width && x == len {
                for i in w_i..w_i + len {
                    bytes[index] = bytes[i];
                    index += 1;
                }
                continue;
            }

            if x >= width {
                bytes[index] = b'\n';
                index += 1;

                x = len;
            } else if x > len {
                bytes[index] = b' ';
                index += 1;

                x += 1;
            }

            for i in w_i..w_i + len {
                bytes[index] = bytes[i];
                index += 1;
            }
        }
    }
    test_log!("!");
    test_log!(bytes);
    test_log!(index);
    bytes.truncate(index);
}

fn bytes_next_word(bytes: &[u8], in_i: &mut usize) -> Option<(usize, usize)> {
    test_log!("next");
    test_log!(bytes);
    let end = bytes.len();
    test_log!(end);
    test_log!(in_i);

    for index in *in_i..end {
        if !is_byte_whitespace(bytes[index]) {
            let out_i = index;
            let mut len = 0;

            for i in index + 1..=end {
                *in_i = i;
                if i == end || is_byte_whitespace(bytes[i]) {
                    len = i - out_i;
                    break;
                }
            }
            if *in_i == end - 1 {
                *in_i = end;
            }

            return Some((out_i, len));
        }
    }
    test_log!("None");
    None
}

pub fn slice_until_first_0<'a>(bytes: &'a [u8]) -> &'a [u8] {
    let mut usable_len = 255;

    for i in 0..bytes.len() {
        if bytes[i] == 0 {
            usable_len = i;
            break;
        }
    }

    if usable_len == 255 {
        bytes
    } else {
        &bytes[..usable_len]
    }
}

// NOTE This does not use a general purpose definition of whitespace.
// This should count a byte as whitespace iff it has all blank
// pixels in this game's font.
#[inline]
pub fn is_byte_whitespace(byte: u8) -> bool {
    let lower_half_byte = byte & 0b0111_1111;
    lower_half_byte < b' '
}

//See NOTE above.
pub fn bytes_split_whitespace<'a>(bytes: &'a [u8]) -> impl Iterator<Item = &'a [u8]> {
    bytes
        .split(|&b| is_byte_whitespace(b))
        .filter(|word| word.len() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use inner_common::test_println;
    use quickcheck::*;

    #[test]
    fn test_bytes_reflow_then_lines_produces_lines_of_the_correct_length() {
        quickcheck(
            bytes_reflow_then_lines_produces_lines_of_the_correct_length
                as fn((Vec<u8>, usize)) -> TestResult,
        )
    }
    fn bytes_reflow_then_lines_produces_lines_of_the_correct_length(
        (s, width): (Vec<u8>, usize),
    ) -> TestResult {
        if width == 0 {
            return TestResult::discard();
        }
        let bytes = &s;

        if byte_reflow_early_out(bytes, width) {
            return TestResult::discard();
        }

        let reflowed = bytes_reflow(bytes, width);
        for line in bytes_lines(&reflowed) {
            assert!(line.len() <= width);
        }

        TestResult::from_bool(true)
    }

    #[test]
    fn test_bytes_reflow_works_for_this_generated_case() {
        let s = vec![27, 0, 27, 0, 27, 0, 27];
        let width = 6;

        let reflowed = bytes_reflow(&s, width);
        if !reflowed.ends_with(&[b'\n', 27]) {
            test_println!("reflowed {:?}", reflowed);
        }
        assert!(reflowed.ends_with(&[b'\n', 27]));
    }
    #[test]
    fn test_bytes_reflow_works_for_this_real_case() {
        let s = vec![
            99, 112, 117, 32, 48, 32, 112, 108, 97, 121, 101, 100, 32, 97, 110, 32, 65, 99, 101,
            32, 111, 102, 32, 104, 101, 97, 114, 116, 115,
        ];
        let width = 28;

        let reflowed = bytes_reflow(&s, width);

        assert!(reflowed.ends_with(&[b'\n', 104, 101, 97, 114, 116, 115]));
    }

    #[test]
    fn test_is_byte_whitespace_works_on_upper_half_values() {
        assert!(is_byte_whitespace(128));
        assert!(is_byte_whitespace(128 + 1));
        assert!(is_byte_whitespace(128 + 32));
        assert!(!is_byte_whitespace(128 + 48));
    }

    #[test]
    fn test_reflow_retains_all_non_whitespace() {
        quickcheck(reflow_retains_all_non_whitespace as fn((String, usize)) -> TestResult)
    }
    fn reflow_retains_all_non_whitespace((s, width): (String, usize)) -> TestResult {
        if width == 0 {
            return TestResult::discard();
        }

        let non_whitespace: String = s.chars().filter(|c| !c.is_whitespace()).collect();

        let reflowed = reflow(&s, width);

        let reflowed_non_whitespace: String =
            reflowed.chars().filter(|c| !c.is_whitespace()).collect();

        assert_eq!(non_whitespace, reflowed_non_whitespace);

        TestResult::from_bool(non_whitespace == reflowed_non_whitespace)
    }

    #[test]
    fn max_length_words_reflow() {
        assert_eq!(
            reflow("1234567890123456789012345 1234567890123456789012345", 25),
            "1234567890123456789012345\n1234567890123456789012345".to_string()
        );
    }

    #[test]
    fn reflow_handles_word_split_at_exactly_the_len() {
        assert_eq!(
            reflow("CPU0, CPU1, CPU2, and you all win.", 25),
            "CPU0, CPU1, CPU2, and you\nall win.".to_string()
        );
    }

    #[test]
    fn bytes_reflow_handles_word_split_just_before_the_len() {
        assert_eq!(
            bytes_reflow(b"CPU 1 played a Queen of clubs.", 28),
            b"CPU 1 played a Queen of\nclubs."
        );
    }

    #[test]
    fn reflow_does_not_add_a_space_if_there_is_no_room() {
        assert_eq!(reflow("12345 67890", 5), "12345\n67890".to_string());
    }

    #[test]
    fn bytes_reflow_does_not_add_a_space_if_there_is_no_room() {
        assert_eq!(bytes_reflow(b"12345 67890", 5), b"12345\n67890");
    }

    // #[test]
    fn test_bytes_reflow_in_place_matches_bytes_reflow() {
        quickcheck(bytes_reflow_in_place_matches_bytes_reflow as fn((Vec<u8>, usize)) -> TestResult)
    }
    fn bytes_reflow_in_place_matches_bytes_reflow((s, width): (Vec<u8>, usize)) -> TestResult {
        if width == 0 {
            return TestResult::discard();
        }
        let mut vec = s.clone();
        let bytes = &mut vec;

        if byte_reflow_early_out(bytes, width) {
            return TestResult::discard();
        }
        test_log!("s");
        test_log!(bytes);
        test_log!(width);
        let copied = bytes_reflow(bytes, width);
        bytes_reflow_in_place(bytes, width);
        let in_place = bytes;
        test_log!(copied);
        test_log!(in_place);
        assert_eq!(copied.len(), in_place.len());
        for (c, i) in copied.iter().zip(in_place) {
            assert_eq!(c, i);
        }

        TestResult::from_bool(true)
    }

    #[test]
    fn test_bytes_reflow_in_place_matches_bytes_reflow_A() {
        let r = bytes_reflow_in_place_matches_bytes_reflow((vec![26], 1));
        assert!(!r.is_failure());
    }

    #[test]
    fn test_bytes_next_word_matches_bytes_split_whitespace() {
        quickcheck(bytes_next_word_matches_bytes_split_whitespace as fn(Vec<u8>) -> TestResult)
    }
    fn bytes_next_word_matches_bytes_split_whitespace(s: Vec<u8>) -> TestResult {
        let bytes = &s;

        let mut split_iter = bytes_split_whitespace(bytes);

        let mut next_i = 0;
        while let Some((w_i, len)) = bytes_next_word(&bytes, &mut next_i) {
            test_log!((w_i, len));
            let word = split_iter.next().unwrap();
            assert_eq!(len, word.len());
            let mut i = w_i;
            for c in word {
                assert_eq!(*c, bytes[i]);
                i += 1;
            }
        }

        TestResult::from_bool(true)
    }

    #[test]
    fn test_bytes_next_word_matches_bytes_split_whitespace_A() {
        let r = bytes_next_word_matches_bytes_split_whitespace(vec![0, 26]);
        assert!(!r.is_failure());
    }

    fn byte_reflow_early_out(bytes: &[u8], width: usize) -> bool {
        bytes.iter().cloned().all(is_byte_whitespace)
            || bytes_split_whitespace(bytes).any(|w| w.len() > width)
    }
}

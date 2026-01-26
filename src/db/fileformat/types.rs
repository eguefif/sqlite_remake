/// A structure representing a variable-length integer (varint) as used in certain binary formats.
/// See 1.6. B-tree Pages in [varint doc](https://www.sqlite.org/fileformat.html)
/// There is a paragraph that starts with variable-length integer after the table.
#[derive(Debug)]
pub struct Varint {
    pub varint: i64, // value of the varint
    pub size: usize, // size in bytes
}

impl Varint {
    pub fn new(buffer: &[u8]) -> Self {
        let mut varint: i64 = 0;
        let mut size = 0;
        for (i, value) in buffer.iter().enumerate().take(9) {
            size += 1;
            if i == 8 {
                varint = (varint << 8) | *value as i64;
                break;
            } else {
                varint = (varint << 7) | (*value & 0b0111_1111) as i64;
                if *value < 0b1000_0000 {
                    break;
                }
            }
        }
        Self { varint, size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_less_than_240() {
        let varint = Varint::new(&vec![0x2B]);
        assert_eq!((43, 1), (varint.varint, varint.size));
    }

    #[test]
    fn test_varint_multi_bytes() {
        let varint = Varint::new(&vec![0x81, 0x47]);
        assert_eq!((43, 1), (varint.varint, varint.size));
    }
    #[test]
    fn read_nine_byte_varint() {
        let varint = Varint::new(&vec![0xff; 9]);
        assert_eq!((43, 1), (varint.varint, varint.size));
    }
}

struct Varint {
    value: i64,
    size: usize,
}

pub fn read_varint(buffer: &[u8]) -> (i64, usize) {
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
                break
            }
        }
    }
    (varint as i64, size)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_less_than_240() {
        assert_eq!((43, 1), read_varint(&vec![0x2B]));
    }

    #[test]
    fn test_varint_multi_bytes() {
        assert_eq!((199, 2), read_varint(&vec![0x81, 0x47]));
    }
     #[test]
    fn read_nine_byte_varint() {
        assert_eq!((-1, 9), read_varint(&vec![0xff; 9]));
    }
}


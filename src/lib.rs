use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid cipher mode")]
    InvalidMode,

    #[error("Invalid cipher key")]
    InvalidKey,

    #[error("Invalid checksum")]
    InvalidChecksum,
}

pub fn decode(output: &mut [u8; 4], input: &[u8]) -> Result<(), Error> {
    let mut fsr = 0;
    let mut context = [0xff, 0xff, 0x00, 0x00];

    for _ in 0..3 {
        context[2] = input[fsr];
        context[3] = input[fsr + 1];
        fsr += 2;

        for _ in 0..16 {
            let temp = context[0];
            let mut c = 0;

            rotate_left(&mut c, &mut context[1]);
            rotate_left(&mut c, &mut context[0]);
            if context[2] & 0x80 == 0x80 {
                context[1] |= 1;
            }
            rotate_left(&mut c, &mut context[3]);
            rotate_left(&mut c, &mut context[2]);
            if (temp & 0x80) == 0x80 {
                context[0] ^= 0x80;
                context[1] ^= 0x05;
            }
        }
    }

    // Verify checksum
    if input[6] != context[0] || input[7] != context[1] {
        return Err(Error::InvalidChecksum);
    }

    if input[1] == 0 || input[1] & 0x20 == 0x20 {
        // TODO: PIC chooses a random value for input[1]
        // Whatever we're given seems just as random, lol!
        return cipher_7(output, input);
    }

    match input[0] {
        1 => cipher_1(output, input),
        2 => cipher_2(),
        3 => cipher_3(),
        4 => cipher_4(),
        5 => cipher_5(),
        6 => cipher_6(),
        7 => cipher_7(output, input),
        0xff => cipher_255(output),
        _ => Err(Error::InvalidMode),
    }
}

fn sbox_0(input: u8) -> Result<u8, Error> {
    #[rustfmt::skip]
    let sbox = [
        0x00, 0x1f, 0x9b, 0x69, 0xa5, 0x80, 0x90, 0xb2,
        0xd7, 0x44, 0xec, 0x75, 0x3b, 0x62, 0x0c, 0xa3,
        0xa6, 0xe4, 0x1f, 0x4c, 0x05, 0xe4, 0x44, 0x6e,
        0xd9, 0x5b, 0x34, 0xe6, 0x08, 0x31, 0x91, 0x72,
    ];

    if input < 32 {
        Ok(sbox[input as usize])
    } else {
        Err(Error::InvalidKey)
    }
}

fn sbox_1(input: u8) -> Result<u8, Error> {
    #[rustfmt::skip]
    let sbox = [
        0x00, 0xae, 0xf3, 0x7b, 0x12, 0xc9, 0x83, 0xf0,
        0xa9, 0x57, 0x50, 0x08, 0x04, 0x81, 0x02, 0x21,
        0x96, 0x09, 0x0f, 0x90, 0xc3, 0x62, 0x27, 0x21,
        0x3b, 0x22, 0x4e, 0x88, 0xf5, 0xc5, 0x75, 0x91,
    ];

    if input < 32 {
        Ok(sbox[input as usize])
    } else {
        Err(Error::InvalidKey)
    }
}

fn sbox_2(input: u8) -> Result<u8, Error> {
    #[rustfmt::skip]
    let sbox = [
        0x00, 0xe3, 0xa2, 0x45, 0x40, 0xe0, 0x09, 0xea,
        0x42, 0x65, 0x1c, 0xc1, 0xeb, 0xb0, 0x69, 0x14,
        0x01, 0xd2, 0x8e, 0xfb, 0xfa, 0x86, 0x09, 0x95,
        0x1b, 0x61, 0x14, 0x0e, 0x99, 0x21, 0xec, 0x40,
    ];

    if input < 32 {
        Ok(sbox[input as usize])
    } else {
        Err(Error::InvalidKey)
    }
}

fn sbox_3(input: u8) -> Result<u8, Error> {
    #[rustfmt::skip]
    let sbox = [
        0x00, 0x25, 0x6d, 0x4f, 0xc5, 0xca, 0x04, 0x39,
        0x3a, 0x7d, 0x0d, 0xf1, 0x43, 0x05, 0x71, 0x66,
        0x82, 0x31, 0x21, 0xd8, 0xfe, 0x4d, 0xc2, 0xc8,
        0xcc, 0x09, 0xa0, 0x06, 0x49, 0xd5, 0xf1, 0x83,
    ];

    if input < 32 {
        Ok(sbox[input as usize])
    } else {
        Err(Error::InvalidKey)
    }
}

fn cipher_1(output: &mut [u8; 4], input: &[u8]) -> Result<(), Error> {
    output[0] = swap(input[2]) ^ sbox_0(input[1])?;
    output[1] = swap(input[3]) ^ sbox_2(input[1])?;
    output[2] = swap(input[4]) ^ sbox_3(input[1])?;
    output[3] = swap(input[5]) ^ sbox_1(input[1])?;

    Ok(())
}

fn cipher_2() -> Result<(), Error> {
    todo!();
}

fn cipher_3() -> Result<(), Error> {
    todo!();
}

fn cipher_4() -> Result<(), Error> {
    todo!();
}

fn cipher_5() -> Result<(), Error> {
    todo!();
}

fn cipher_6() -> Result<(), Error> {
    todo!();
}

fn cipher_7(output: &mut [u8; 4], input: &[u8]) -> Result<(), Error> {
    if input[1] & 1 == 0 {
        cipher_1(output, input)
    } else {
        output[0] = !input[2];
        output[1] = !input[3];
        output[2] = !input[4];
        output[3] = !input[5];

        Ok(())
    }
}

fn cipher_255(output: &mut [u8; 4]) -> Result<(), Error> {
    output[0] = 0;
    output[1] = 0;
    output[2] = 1;
    output[3] = 2;

    Ok(())
}

fn rotate_left(c: &mut u8, data: &mut u8) {
    let temp = *data >> 7;
    let output = (*data << 1) | *c;

    *c = temp;
    *data = output;
}

fn swap(data: u8) -> u8 {
    data << 4 | data >> 4
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rotate_left() {
        let mut c = 0;
        let mut data = 0xff;

        rotate_left(&mut c, &mut data);
        assert_eq!(c, 1);
        assert_eq!(data, 0xfe);

        c = 0;
        data = 0x55;

        rotate_left(&mut c, &mut data);
        assert_eq!(c, 0);
        assert_eq!(data, 0xaa);

        c = 1;
        data = 0xaa;

        rotate_left(&mut c, &mut data);
        assert_eq!(c, 1);
        assert_eq!(data, 0x55);
    }

    #[test]
    fn test_swap() {
        assert_eq!(swap(0x7f), 0xf7);
        assert_eq!(swap(0xa5), 0x5a);
        assert_eq!(swap(0x94), 0x49);
    }
}

# picard

![picard](./img/picard.jpg)

Make it so!

## What is it?

The PS1 Action Replay Pro/GameShark Pro contains a PIC12C508 microcontroller that serves as a copy protection device. This project reimplements the functionality of the PID on these cheat cartridges. It can be used as a library, or a standalone command line application. The library currently depends on the Rust standard library (for the `Error` trait), but can very easily be made `no_std` compatible to run on microcontrollers if needed.

## Why?

Nostalgia, I guess! While reverse engineering the N64 AR Pro/GS Pro, one of the empty pads on the PCB was determined to be meant for a PIC. Even though we haven't seen any code in the firmware that expects it to exist. We were curious what its intended purpose might have been. We speculated it may have been a copy protection device of some sort. The only evidence is that the PIC was obviously not required for the device to operate, and the way it is connected suggests that the firmware would have been able to communicate with it, and nothing more.

To verify our suspicion that the PIC was for copy protection, we turned our attention to the PS1 AR Pro/GS Pro. The PIC on this device is wired very similarly to the unoccupied pad on the N64 AR Pro/GS Pro.

## Technical description

The PIC is connected to programmable logic on the PCB, which when decoded reveals that the firmware communicates with the PIC through two hardware registers:

- `0x1f600030`: (Write only) Send PIC requests and set bank switching controller
- `0x1f600038`: (Read only) Receive PIC responses

The `0x1f600030` serves a dual purpose, where writes can bank switch the SRAM and EEPROMs and also provide the serial bus for PIC requests.

| Register / Bits | 8..4 | 3             | 2            | 1         | 0           |
|-----------------|------|---------------|--------------|-----------|-------------|
| `0x1f600030`    | NC   | Request clock | Request data | SRAM bank | EEPROM bank |

| Register / Bits | 8..2 | 1              | 0             |
|-----------------|------|----------------|---------------|
| `0x1f600038`    | NC   | Response clock | Response data |

Notes:

- NC: No Connection.
- SRAM and EEPROM both have only two banks, so they are fully accessible with 1 bit each.

### Enabling the PIC

The PIC starts in a "waiting" state, where it will do nothing until it receives a specific sequence of bits on its request data line. The sequence is 8 bytes with value `0xae`. The AR firmware sends this sequence repeatedly until the clock on register `0x1f600038` goes high, signaling the PIC has entered its "request-response" state.

### Request format

With the PIC in its "request-response" state, it will begin waiting for 8-byte requests and sending back 4-byte responses. The requests use this format, where bytes are numbered most significant first (reversed compared to array indexing):

| Byte | Description |
|------|-------------|
| 7    | Cipher mode |
| 6    | Cipher key  |
| 5..2 | Cipher text |
| 1..0 | Checksum    |

The checksum is implemented with a pair of 16-bit [LFSRs](https://en.wikipedia.org/wiki/Linear-feedback_shift_register) (which I've arbitrarily named `context` and combined into a single `u8` array).

After the checksum has been validated, the cipher mode selects a cipher function to apply to the cipher text.

### Cipher modes

Eight cipher modes are supported (there is code for an unused ninth mode). Any invalid cipher mode received will cause the PIC to enter its "terminated state" where it asserts GP2 (unconnected on the PCB) and halts until reset.

| Mode | Description        |
|------|--------------------|
| 1    | `S(k) ^ swap(t)`   |
| 2    | `S(k) + t`         |
| 3    | `S(k) - t`         |
| 4    | `S(k) ^ t + S(k)`  |
| 5    | `S(k) - t ^ S'(k)` |
| 6    | `S(k++) +- t`      |
| 7    | See note           |
| 255  | ID or version      |

Notes:

- `k` is the cipher key.
- `t` is one byte of the cipher text.
- `S()` is a function that substitutes its input with deterministic output. (Common substitution box.) There are four substitution boxes, and which one is used depends on the cipher mode and which byte of the cipher text is being processed.
- `swap()` is a function that swaps the nibble in the given byte. E.g. `swap(0xf2) == 0x2f`
- `^` is XOR, `+` is ADD, `-` is SUBTRACT, `++` is post-increment, `+-` means either ADD or SUBTRACT depending on which byte in the cipher text is being accessed. See code for details.
- Cipher mode 7 will defer to cipher mode 1 with 50% probability (depending on the LSB of the cipher key) or will use `~t` inverting the bits of the cipher text byte.
- ID or version has an unknown purpose. Cipher mode 255 always returns the same fixed response: `0x00 0x00 0x01 0x02`. This may be an identifier or version number.

### Response format

Immediately after a request has been processed, the PIC puts its response on its serial output.

Responses are simply the 4 bytes of plain text produced by the cipher mode operation. After the firmware reads the last byte, the PIC goes back to waiting for the next request.

### Known requests

The firmware is currently only known to send requests with cipher mode 1 and cipher key 2. Approximately 8 or 9 seconds after boot up, the firmware sends exactly 200 requests with these settings. It is currently unknown what the purpose of the data is.

## Credits

This work could not have been done without @modman, who meticulously documented the hardware with a full schematic and programmable logic dumps. And most importantly, decapped and dumped the PIC12C508. This source code is my (@parasyte) attempt at interpreting the PIC assembly and hardware schematics.

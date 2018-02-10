use super::*;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::io::Cursor;

struct Pipe(Sender<u8>, Receiver<u8>, Vec<u8>);

fn pipe() -> (Pipe, Pipe) {
    let ((tx1, rx1), (tx2, rx2)) = (channel(), channel());
    (Pipe(tx1, rx2, vec![]), Pipe(tx2, rx1, vec![]))
}

impl io::Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for i in 0..buf.len() {
            match self.1.recv() {
                Ok(byte) => buf[i] = byte,
                Err(_) => return Ok(i)
            }
        }

        Ok(buf.len())
    }
}

impl io::Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        buf.iter().for_each(|b| self.2.push(*b));
        for (i, byte) in buf.iter().cloned().enumerate() {
            if let Err(e) = self.0.send(byte) {
                eprintln!("Write error: {}", e);
                return Ok(i);
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_loop() {
    let mut input = [0u8; 384];
    for (i, chunk) in input.chunks_mut(128).enumerate() {
        chunk.iter_mut().for_each(|b| *b = i as u8);
    }

    let (tx, rx) = pipe();
    let tx_thread = std::thread::spawn(move || Xmodem::transmit(&input[..], rx));
    let rx_thread = std::thread::spawn(move || {
        let mut output = [0u8; 384];
        Xmodem::receive(tx, &mut output[..]).map(|_| output)
    });

    assert_eq!(tx_thread.join().expect("tx join okay").expect("tx okay"), 384);
    let output = rx_thread.join().expect("rx join okay").expect("rx okay");
    assert_eq!(&input[..], &output[..]);
}

#[test]
fn read_byte() {
    let byte = Xmodem::new(Cursor::new(vec![CAN]))
        .read_byte(false)
        .expect("read a byte");

    assert_eq!(byte, CAN);

    let e = Xmodem::new(Cursor::new(vec![CAN]))
        .read_byte(true)
        .expect_err("abort on CAN");

    assert_eq!(e.kind(), io::ErrorKind::ConnectionAborted);
}

#[test]
fn test_expect_byte() {
    let mut xmodem = Xmodem::new(Cursor::new(vec![1, 1]));
    assert_eq!(xmodem.expect_byte(1, "1").expect("expected"), 1);
    let e = xmodem.expect_byte(2, "1, please").expect_err("expect the unexpected");
    assert_eq!(e.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn test_expect_byte_or_cancel() {
    let mut buffer = vec![2, 0];
    let b = Xmodem::new(Cursor::new(buffer.as_mut_slice()))
        .expect_byte_or_cancel(2, "it's a 2")
        .expect("got a 2");

    assert_eq!(b, 2);
}

#[test]
fn test_expect_can() {
    let mut xmodem = Xmodem::new(Cursor::new(vec![CAN]));
    assert_eq!(xmodem.expect_byte(CAN, "hi").expect("CAN"), CAN);
}

#[test]
fn test_unexpected_can() {
    let e = Xmodem::new(Cursor::new(vec![CAN]))
        .expect_byte(SOH, "want SOH")
        .expect_err("have CAN");

    assert_eq!(e.kind(), io::ErrorKind::ConnectionAborted);
}

#[test]
fn test_cancel_on_unexpected() {
    let mut buffer = vec![CAN, 0];
    let e = Xmodem::new(Cursor::new(buffer.as_mut_slice()))
        .expect_byte_or_cancel(SOH, "want SOH")
        .expect_err("have CAN");

    assert_eq!(e.kind(), io::ErrorKind::ConnectionAborted);
    assert_eq!(buffer[1], CAN);

    let mut buffer = vec![0, 0];
    let e = Xmodem::new(Cursor::new(buffer.as_mut_slice()))
        .expect_byte_or_cancel(SOH, "want SOH")
        .expect_err("have 0");

    assert_eq!(e.kind(), io::ErrorKind::InvalidData);
    assert_eq!(buffer[1], CAN);
}

#[test]
fn test_can_in_packet_and_checksum() {
    let mut input = [0u8; 256];
    input[0] = CAN;

    let (tx, rx) = pipe();
    let tx_thread = std::thread::spawn(move || Xmodem::transmit(&input[..], rx));
    let rx_thread = std::thread::spawn(move || {
        let mut output = [0u8; 256];
        Xmodem::receive(tx, &mut output[..]).map(|_| output)
    });

    assert_eq!(tx_thread.join().expect("tx join okay").expect("tx okay"), 256);
    let output = rx_thread.join().expect("rx join okay").expect("rx okay");
    assert_eq!(&input[..], &output[..]);
}

#[test]
fn test_transmit_reported_bytes() {
    let (input, mut output) = ([0u8; 50], [0u8; 128]);
    let (tx, rx) = pipe();
    let tx_thread = std::thread::spawn(move || Xmodem::transmit(&input[..], rx));
    let rx_thread = std::thread::spawn(move || Xmodem::receive(tx, &mut output[..]));
    assert_eq!(tx_thread.join().expect("tx join okay").expect("tx okay"), 50);
    assert_eq!(rx_thread.join().expect("rx join okay").expect("rx okay"), 128);
}

#[test]
fn test_raw_transmission() {
    let mut input = [0u8; 256];
    let mut output = [0u8; 256];
    (0..256usize).into_iter().enumerate().for_each(|(i, b)| input[i] = b as u8);

    let (mut tx, mut rx) = pipe();
    let tx_thread = std::thread::spawn(move || {
        Xmodem::transmit(&input[..], &mut rx).expect("transmit okay");
        rx.2
    });

    let rx_thread = std::thread::spawn(move || {
        Xmodem::receive(&mut tx, &mut output[..]).expect("receive okay");
        tx.2
    });

    let rx_buf = tx_thread.join().expect("tx join okay");
    let tx_buf = rx_thread.join().expect("rx join okay");

    // check packet 1
    assert_eq!(&rx_buf[0..3], &[SOH, 1, 255 - 1]);
    assert_eq!(&rx_buf[3..(3 + 128)], &input[..128]);
    assert_eq!(rx_buf[131], input[..128].iter().fold(0, |a: u8, b| a.wrapping_add(*b)));

    // check packet 2
    assert_eq!(&rx_buf[132..135], &[SOH, 2, 255 - 2]);
    assert_eq!(&rx_buf[135..(135 + 128)], &input[128..]);
    assert_eq!(rx_buf[263], input[128..].iter().fold(0, |a: u8, b| a.wrapping_add(*b)));

    // check EOT
    assert_eq!(&rx_buf[264..], &[EOT, EOT]);

    // check receiver responses
    assert_eq!(&tx_buf, &[NAK, ACK, ACK, NAK, ACK]);
}

#[test]
fn test_small_packet_eof_error() {
    let mut xmodem = Xmodem::new(Cursor::new(vec![NAK, NAK, NAK]));

    let mut buffer = [1, 2, 3];
    let e = xmodem.read_packet(&mut buffer[..]).expect_err("read EOF");
    assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof);

    let e = xmodem.write_packet(&buffer).expect_err("write EOF");
    assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof);
}

#[test]
fn test_bad_control() {
    let mut packet = [0; 128];
    let e = Xmodem::new(Cursor::new(vec![0, CAN]))
        .read_packet(&mut packet[..])
        .expect_err("CAN");

    assert_eq!(e.kind(), io::ErrorKind::ConnectionAborted);

    let e = Xmodem::new(Cursor::new(vec![0, 0xFF]))
        .read_packet(&mut packet[..])
        .expect_err("bad contorl");

    assert_eq!(e.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn test_eot() {
    let mut buffer = vec![NAK, 0, NAK, 0, ACK];
    Xmodem::new(Cursor::new(buffer.as_mut_slice()))
        .write_packet(&[])
        .expect("write empty buf for EOT");

    assert_eq!(&buffer[..], &[NAK, EOT, NAK, EOT, ACK]);
}

#[test]
fn test_read_packet() {
    use std::io::{Read, Write};

    let (mut tx, rx) = pipe();
    let source_packet = [0u8; 128];

    let rx_thread = std::thread::spawn(move || {
        let mut xmodem = Xmodem::new_with_progress(rx, |progress| {
            if let Progress::Packet(number) = progress {
                assert_eq!(number, 1);
            } else if let Progress::Started = progress {
                // Valid.
            } else {
                assert!(false);
            }
        });

        let mut dest_packet = [0u8; 128];
        xmodem.read_packet(&mut dest_packet).expect("read packet");
        assert_eq!(&dest_packet[..], &source_packet[..]);

        assert_eq!(xmodem.read_packet(&mut dest_packet).ok(), Some(0));
    });

    let mut start = [0u8; 1];
    tx.read(&mut start).expect("start");
    assert_eq!(&start[..], &[NAK]);

    // Packet header.
    tx.write(&[SOH, 1, 254]).expect("header");

    tx.write(&source_packet).expect("packet");
    tx.write(&[0]).expect("checksum");

    // We should receive an ACK to indicate it was properly received.
    let mut ack = [0u8; 1];
    tx.read(&mut ack).expect("ack");
    assert_eq!(&ack[..], &[ACK]);

    // Send the first EOT, expect a NAK back.
    tx.write(&[EOT]).expect("eot 1");

    let mut nak = [0u8; 1];
    tx.read(&mut nak).expect("nak");
    assert_eq!(&nak[..], &[NAK]);

    // Send the second EOT, and expect an ACK to finish the sequence.
    tx.write(&[EOT]).expect("eot 2");

    let mut eot_ack = [0u8; 1];
    tx.read(&mut eot_ack).expect("eot ack");
    assert_eq!(&eot_ack[..], &[ACK]);

    rx_thread.join().expect("rx join okay");
}

/// Test sending a packet containing control characters, it should be received
/// intact.
#[test]
fn test_read_packet_control_characters() {
    use std::io::{Read, Write};

    let (mut tx, rx) = pipe();
    let mut xmodem = Xmodem::new(rx);

    let mut source_packet = [0u8; 128];
    source_packet[0..5].copy_from_slice(&[SOH, EOT, ACK, NAK, CAN]);
    let checksum: u8 = source_packet.iter().fold(0, |acc, &x| {
        acc.wrapping_add(x)
    });

    tx.write(&[SOH, 1, 254]).expect("header");
    tx.write(&source_packet).expect("packet");
    tx.write(&[checksum]).expect("checksum");

    let mut dest_packet = [0u8; 128];
    xmodem.read_packet(&mut dest_packet).expect("read packet");
    assert_eq!(&dest_packet[..], &source_packet[..]);

    // We should get the NAK to indicate start of transmission, and then an ACK
    // to indicate success.
    let mut response = [0u8; 2];
    tx.read(&mut response).expect("response");
    assert_eq!(&response[..], &[NAK, ACK]);
}

/// Test sending with CAN as a checksum.
#[test]
fn test_read_packet_checksum_can() {
    use std::io::{Read, Write};

    let (mut tx, rx) = pipe();
    let mut xmodem = Xmodem::new(rx);

    let mut source_packet = [0u8; 128];
    source_packet[0] = CAN;

    tx.write(&[SOH, 1, 254]).expect("header");
    tx.write(&source_packet).expect("packet");
    tx.write(&[CAN]).expect("checksum");

    let mut dest_packet = [0u8; 128];
    xmodem.read_packet(&mut dest_packet).expect("read packet");
    assert_eq!(&dest_packet[..], &source_packet[..]);

    // We should get the NAK to indicate start of transmission, and then an ACK
    // to indicate success.
    let mut response = [0u8; 2];
    tx.read(&mut response).expect("response");
    assert_eq!(&response[..], &[NAK, ACK]);
}

/// Test large packet numbers and wrapping.
#[test]
fn test_read_packet_numbers() {
    use std::io::{Read, Write};

    let (mut tx, rx) = pipe();
    let mut xmodem = Xmodem::new(rx);

    let mut started = false;

    for x in 1..512 {
        let mut source_packet = [0u8; 128];

        let packet_number: u8 = x as u8;
        tx.write(&[SOH, packet_number, (255 - packet_number)]).expect("header");
        tx.write(&source_packet).expect("packet");
        tx.write(&[0]).expect("checksum");

        let mut dest_packet = [0u8; 128];
        xmodem.read_packet(&mut dest_packet).expect("read packet");
        assert_eq!(&dest_packet[..], &source_packet[..]);

        if !started {
            started = true;

            let mut start = [0u8; 1];
            tx.read(&mut start).expect("start");
            assert_eq!(&start[..], &[NAK]);
        }

        // We should get an ACK to indicate success.
        let mut ack = [0u8; 1];
        tx.read(&mut ack).expect("ack");
        assert_eq!(&ack[..], &[ACK]);
    }

    // Don't validate the rest it's handled by test_read_packet.
}

/// Test Xmodem::write_packet in isolation, sends one packet and verifies that
/// the transmission is initiated per the protocol, and then finishes the
/// transfer.
#[test]
fn test_write_packet() {
    use std::io::{Read, Write};

    let (tx, mut rx) = pipe();
    let mut xmodem = Xmodem::new_with_progress(tx, |progress| {
        if let Progress::Packet(number) = progress {
            assert_eq!(number, 1);
        } else if let Progress::Waiting = progress {
            // Valid.
        } else {
            assert!(false);
        }
    });

    // NAK to start transmission, ACK to accept packet to unblock write_packet.
    rx.write(&[NAK, ACK]).expect("start");

    let source_packet = [0u8; 128];
    xmodem.write_packet(&source_packet).expect("write packet");

    let mut header = [0u8; 3];
    rx.read(&mut header).expect("read header");
    assert_eq!(&header[..], &[SOH, 1, 254]);

    let mut dest_packet = [0u8; 128];
    rx.read(&mut dest_packet).expect("read packet");
    assert_eq!(&dest_packet[..], &source_packet[..]);

    let mut checksum = [0u8; 1];
    rx.read(&mut checksum).expect("checksum");
    assert_eq!(&checksum[..], &[0]);

    // EOT sequence to unblock write_packet.
    rx.write(&[NAK, ACK]).expect("write eot sequence");
    xmodem.write_packet(&[]).expect("transmission end");

    let mut eot = [0u8; 2];
    rx.read(&mut eot).expect("eot response");
    assert_eq!(&eot[..], &[EOT, EOT]);
}

/// Test sending a packet containing control characters, it should be received
/// intact.
#[test]
fn test_write_packet_control_characters() {
    use std::io::{Read, Write};

    let (tx, mut rx) = pipe();
    let mut xmodem = Xmodem::new(tx);

    // All the bytes sent from the receiver->transmitter, including start
    // of transmission, packet ack, and EOT sequence response.
    rx.write(&[NAK, ACK, NAK, ACK]).expect("responses");

    let mut source_packet = [0u8; 128];
    source_packet[0..5].copy_from_slice(&[SOH, EOT, ACK, NAK, CAN]);
    xmodem.write_packet(&source_packet).expect("write packet");
    xmodem.write_packet(&[]).expect("transmission end");

    let mut header = [0u8; 3];
    rx.read(&mut header).expect("read header");
    assert_eq!(&header[..], &[SOH, 1, 254]);

    let mut dest_packet = [0u8; 128];
    rx.read(&mut dest_packet).expect("read packet");
    assert_eq!(&dest_packet[..], &source_packet[..]);

    // Don't validate the rest of the data, that is tested in test_write_packet.
}

#[test]
fn test_write_packet_numbers() {
    use std::io::{Read, Write};

    let (tx, mut rx) = pipe();
    let mut xmodem = Xmodem::new(tx);

    // All the bytes sent from the receiver->transmitter, including start
    // of transmission, packet ack, and EOT sequence response.
    rx.write(&[NAK]).expect("start");

    for x in 1..512 {
        rx.write(&[ACK]).expect("packet ack");

        let mut source_packet = [0u8; 128];
        xmodem.write_packet(&source_packet).expect("write packet");

        let mut header = [0u8; 3];
        rx.read(&mut header).expect("read header");
        let packet_number: u8 = x as u8;
        assert_eq!(&header[..], &[SOH, packet_number as u8, (255 - packet_number) as u8]);

        let mut dest_packet = [0u8; 128];
        rx.read(&mut dest_packet).expect("read packet");
        assert_eq!(&dest_packet[..], &source_packet[..]);

        let mut checksum = [0u8; 1];
        rx.read(&mut checksum).expect("read checksum");
    }

    // Don't validate the rest, that's handled by test_write_packet.
}

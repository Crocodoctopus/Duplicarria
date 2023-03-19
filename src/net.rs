use std::net::{SocketAddr, UdpSocket};

use crate::game::net::*;

const PACKET_MAX_SIZE: usize = 576;

pub fn send(socket: &UdpSocket, events: &[NetEvent]) -> usize {
    let dst = socket.peer_addr().unwrap();
    send_to(socket, dst, events)
}

pub fn send_to(socket: &UdpSocket, dst: SocketAddr, events: &[NetEvent]) -> usize {
    let mut packet = [0u8; PACKET_MAX_SIZE];
    let mut packet_size = 0;
    let mut sent = 0;

    use bincode::serialize_into;
    use bincode::serialized_size;

    for event in events {
        // Get serialized event size
        let event_size = serialized_size(&event).unwrap() as usize;

        // If the event can't fit in the current packet, send the packet
        if packet_size + event_size > PACKET_MAX_SIZE {
            socket.send_to(&packet[..packet_size], dst);
            sent += packet_size;
            packet_size = 0;
        }

        // Pack net event into packet
        serialize_into(&mut packet[packet_size..], &event);
        packet_size += event_size;
    }

    // Send last packet if needed
    if packet_size != 0 {
        socket.send_to(&packet[..packet_size], dst);
        sent += packet_size;
    }

    return sent;
}

pub fn recv(socket: &UdpSocket, vec: &mut Vec<NetEvent>) -> usize {
    let mut packet = [0u8; PACKET_MAX_SIZE];
    let mut received = 0;

    use std::io::BufReader;

    // While there are packets...
    while let Ok((n, _)) = socket.recv_from(&mut packet) {
        let mut reader = BufReader::new(&packet[..n]);
        received += n;
        // While there is still data to deserialize
        while let Ok(event) = bincode::deserialize_from::<_, NetEvent>(&mut reader) {
            vec.push(event);
        }
    }

    return received;
}

pub fn recv_from(socket: &UdpSocket, vec: &mut Vec<(NetEvent, SocketAddr)>) -> usize {
    let mut packet = [0u8; PACKET_MAX_SIZE];
    let mut received = 0;

    use std::io::BufReader;

    // While there are packets...
    while let Ok((n, src)) = socket.recv_from(&mut packet) {
        let mut reader = BufReader::new(&packet[..n]);
        received += n;
        // While there is still data to deserialize
        while let Ok(event) = bincode::deserialize_from::<_, NetEvent>(&mut reader) {
            vec.push((event, src));
        }
    }

    return received;
}

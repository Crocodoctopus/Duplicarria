use std::net::{SocketAddr, UdpSocket};

use crate::shared::net_event::*;

const PACKET_MAX_SIZE: usize = 576;

pub fn send(socket: &UdpSocket, events: impl IntoIterator<Item = NetEvent>) {
    let dst = socket.peer_addr().unwrap();
    send_to(socket, dst, events)
}

pub fn send_to(socket: &UdpSocket, dst: SocketAddr, events: impl IntoIterator<Item = NetEvent>) {
    let mut packet = [0u8; PACKET_MAX_SIZE];
    let mut packet_size = 0;

    use bincode::serialize_into;
    use bincode::serialized_size;

    for event in events {
        // Get serialized event size
        let event_size = serialized_size(&event).unwrap() as usize;

        // If the event can't fit in the current packet, send the packet
        if packet_size + event_size > PACKET_MAX_SIZE {
            socket.send_to(&packet[..packet_size], dst);
            packet_size = 0;
        }

        // Pack net event into packet
        serialize_into(&mut packet[packet_size..], &event);
        packet_size += event_size;
    }

    // Send last packet if needed
    if packet_size != 0 {
        socket.send_to(&packet[..packet_size], dst);
    }
}

pub fn recv(socket: &UdpSocket) -> Vec<NetEvent> {
    let mut events = Vec::new();
    let mut packet = [0u8; PACKET_MAX_SIZE];

    use std::io::BufReader;

    // While there are packets...
    while let Ok((n, _)) = socket.recv_from(&mut packet) {
        let mut reader = BufReader::new(&packet[..n]);
        // While there is still data to deserialize
        while let Ok(event) = bincode::deserialize_from::<_, NetEvent>(&mut reader) {
            events.push(event);
        }
    }

    events
}

pub fn recv_from(socket: &UdpSocket) -> Vec<(NetEvent, SocketAddr)> {
    let mut events = Vec::new();
    let mut packet = [0u8; PACKET_MAX_SIZE];

    use std::io::BufReader;

    // While there are packets...
    while let Ok((n, src)) = socket.recv_from(&mut packet) {
        let mut reader = BufReader::new(&packet[..n]);
        // While there is still data to deserialize
        while let Ok(event) = bincode::deserialize_from::<_, NetEvent>(&mut reader) {
            events.push((event, src));
        }
    }

    events
}

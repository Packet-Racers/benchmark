use std::io::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct Udp {}

impl Udp {
  pub async fn send_file(
    file_data: Vec<u8>,
    receiver_address: &SocketAddr,
    packet_size: usize,
    socket: &UdpSocket,
  ) -> Result<()> {

    let packets = file_data.chunks(packet_size).enumerate();

    // println!("Sending {} packets of size {}", packets.len(), packet_size);

    // Send each packet to the receiver
    for (_packet_num, packet) in packets {
      // println!("[UDP]: Sending packet {}...", packet_num + 1);
      socket.send_to(packet, receiver_address).await?;
    }

    // send 0 bytes to signal end of transmission
    socket.send_to(&[], receiver_address).await?;

    Ok(())
  }

  pub async fn receive_file(
    socket: &UdpSocket,
    packet_size: usize,
  ) -> Result<Vec<u8>> {
    let mut received_data = Vec::new();
    loop {
      let mut buffer = vec![0u8; packet_size];
      let (number_of_bytes, _addr) = socket.recv_from(&mut buffer).await.unwrap();
      // println!("[UDP]: Received {} bytes from {}", number_of_bytes, addr);

      if number_of_bytes == 0 {
        break;
      }

      received_data.extend_from_slice(&buffer[..number_of_bytes]);
    }

    Ok(received_data)
  }
}

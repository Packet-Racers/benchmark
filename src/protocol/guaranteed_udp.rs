use std::io::Result;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

const PREAMBLE_SIZE: usize = 8;

const ACK_TIMEOUT: Duration = Duration::from_secs(1);

pub struct GuaranteedUdp {}

impl GuaranteedUdp {
  pub async fn send_file(
    file_data: Vec<u8>,
    receiver_address: &SocketAddr,
    packet_size: usize,
    socket: &UdpSocket,
  ) -> Result<()> {
    // let mut file = File::open(file_name)?;
    // let mut file_data = Vec::new();
    // file.read_to_end(&mut file_data)?;

    let packets = file_data.chunks(packet_size).enumerate();

    // println!("Sending {} packets of size {}", packets.len(), packet_size);

    // Send each packet to the receiver
    for (packet_num, packet) in packets {
      let mut retries = 0;
      let mut ack_received = false;

      // Create a new packet with the packet number as the first byte
      // let mut modified_packet = vec![packet_num as u8];
      let mut modified_packet = packet_num.to_be_bytes().to_vec();
      modified_packet.extend_from_slice(packet);

      while !ack_received && retries < 10 {
        // println!(
        //   "[UDP]: Sending packet {} of size {} (attempt {})",
        //   packet_num + 1,
        //   modified_packet.len(),
        //   retries + 1
        // );
        socket.send_to(&modified_packet, receiver_address).await?;

        // Wait for an acknowledgment packet (1 byte) or a timeout
        let mut ack_buffer = [0u8; 4];

        if let Ok(Ok(recv_len)) = timeout(ACK_TIMEOUT, socket.recv(&mut ack_buffer)).await {
          if recv_len == 4 && &ack_buffer == b"ACK\n" {
            ack_received = true;
          }
        }

        if !ack_received {
          println!(
            "[UDP]: Sending packet {} (attempt {})",
            packet_num + 1,
            retries + 1
          );
        }

        retries += 1;
      }

      if !ack_received {
        return Err(std::io::Error::new(
          std::io::ErrorKind::TimedOut,
          format!(
            "Failed to send packet {} after multiple attempts",
            packet_num + 1
          ),
        ));
      }
    }

    // send 0 bytes to signal end of transmission
    socket.send_to(&[], receiver_address).await?;

    Ok(())
  }

  pub async fn receive_file(socket: &UdpSocket, packet_size: usize) -> Result<Vec<u8>> {
    let mut received_packets: Vec<Vec<u8>> = Vec::new();

    loop {
      let mut buffer = vec![0; packet_size + PREAMBLE_SIZE];
      let (number_of_bytes, sender_address) = socket.recv_from(&mut buffer).await?;

      if number_of_bytes == 0 {
        break; // End of transmission
      }

      // Extract the packet number from the first byte of the packet
      let packet_num_bytes: [u8; PREAMBLE_SIZE] = buffer[..PREAMBLE_SIZE].try_into().unwrap();
      let packet_num = usize::from_be_bytes(packet_num_bytes);

      // let packet_num = buffer[0] as usize;
      let packet_data = &buffer[PREAMBLE_SIZE..number_of_bytes];
      // println!("[UDP]: Received packet {}", packet_num);

      // Insert the packet into the received_packets vector
      if packet_num <= received_packets.len() {
        received_packets.insert(packet_num, packet_data.to_vec());
      } else {
        let padding_size = packet_num - received_packets.len();
        received_packets.extend(vec![vec![]; padding_size]);
        received_packets.push(packet_data.to_vec());
      }

      // Send acknowledgment packet
      socket.send_to(b"ACK\n", sender_address).await?;
    }

    // Sort the received packets by packet number and construct the sorted data array
    let mut sorted_data: Vec<u8> = Vec::new();
    for packet in received_packets {
      sorted_data.extend(packet);
    }

    println!("File received successfully.");
    Ok(sorted_data)
  }
}

use async_std::io::{WriteExt, ReadExt};
use async_std::net::TcpStream;
use async_std::sync::{Mutex, Arc};
use async_std::{task, io, net::TcpListener};
use async_std::stream::StreamExt;


async fn on_connection(mut stream: TcpStream, connections: Arc<Mutex<Vec<TcpStream>>> ) -> io::Result<()>{
    let addr = stream.peer_addr()?;
    println!("New Connection: {}", addr);

    let mut buffer = [0u8; 1024];
    loop {
        let len = stream.read(&mut buffer).await?;
        if len > 0 {
            println!("{}", String::from_utf8_lossy(&buffer[..len]));

            let connections_guard = connections.lock().await;
            for mut client in connections_guard.iter() {
                if client.peer_addr()? != stream.peer_addr()? {
                    client.write(&buffer).await?;
                }
            }
        } else {
            println!("Disconnected: {}", stream.peer_addr()?);
            let mut connections_guard = connections.lock().await;
            
            let client_index = connections_guard.iter().position(|x| (*x).peer_addr().unwrap() == stream.peer_addr().unwrap()).unwrap();
            connections_guard.remove(client_index);

            break
        }
    }

    Ok(())
    
}

#[async_std::main]
async fn main() -> io::Result<()>{
    let connections: Vec<TcpStream> = vec![];
    let connections = Arc::new(Mutex::new(connections));
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream?;

        let connections = connections.clone();
        let mut write_permission = connections.lock().await;
        write_permission.push(stream.clone());
        drop(write_permission);

        task::spawn(on_connection(stream, connections));
    }

    Ok(())
}
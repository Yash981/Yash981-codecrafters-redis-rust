#![allow(unused_imports)]
use tokio::net::{TcpListener,TcpStream};
use anyhow::Result;
use resp::Value;

mod resp;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    //
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {
                    handle_connection(stream).await
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
async fn handle_connection(stream:TcpStream) {
    let mut handler = resp::RespHandler::new(stream);

    println!("Starting read loop");

    loop {
        let value = handler.read_value().await.unwrap();
        println!("Got value {:?}",value);
        let response = if let Some(v) = value {
            let (command,args) = extract_command(v).unwrap();
            match command.as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                c => panic!("Cannot handle command {}", c),
            }

        } else {
            break;
        };

        println!("Sending value {:?}",response);

        handler.write_value(response).await.unwrap();
    }
}
fn extract_command(value:Value) -> Result<(String,Vec<Value>)> {
    match value {
        Value::Array(a) => {
            Ok((
                unpack_bulk_str(a.first().unwrap().clone())?,
                a.into_iter().skip(1).collect(),
            ))
        },
        _ => Err(anyhow::anyhow!("Unexpected cmd format")),
    }
}

fn unpack_bulk_str(value:Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected command to be a bulk string"))
    }
}
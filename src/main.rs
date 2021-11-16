// main.rs

// use anyhow::anyhow;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use serde_json::Value;
// use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let mut mqttoptions = MqttOptions::new("mqtt2coap", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    client.subscribe("zigbee2mqtt/leikkimokki", QoS::AtMostOnce)?;

    // Iterate to poll the eventloop for connection progress
    for (i, notification) in connection.iter().enumerate() {
        let event = notification?;
        // println!("Notification #{} = {:?}", i, event);
        match event {
            Event::Incoming(ev) => match ev {
                Packet::Publish(p) => {
                    println!("Publish #{} = {:?}", i, p);
                    let msg = String::from_utf8_lossy(&p.payload);
                    println!("Payload: {}", msg);
                    let json: Value = serde_json::from_str(&msg)?;
                    println!("Json: {:?}", json);
                }
                _ => {}
            },
            Event::Outgoing(_) => {}
        }
    }
    Ok(())
}
// EOF

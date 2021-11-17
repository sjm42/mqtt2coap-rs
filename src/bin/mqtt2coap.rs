// main.rs

// use anyhow::anyhow;
use coap::CoAPClient;
use log::*;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use serde_json::json;
use serde_json::Value;
use std::time::Duration;
use structopt::StructOpt;

use mqtt2coap::*;

fn main() -> anyhow::Result<()> {
    let mut opts = OptsCommon::from_args();
    opts.finish()?;
    start_pgm(&opts, "mqtt2coap");

    let mut mqttoptions = MqttOptions::new("mqtt2coap", &opts.mqtt_host, opts.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    for topic in opts.topics.split(',') {
        client.subscribe(format!("zigbee2mqtt/{}", topic), QoS::AtMostOnce)?;
    }

    // Iterate to poll the eventloop for connection progress
    for (i, notification) in connection.iter().enumerate() {
        let event = notification?;
        match &event {
            Event::Incoming(ev) => {
                if let Packet::PingResp = *ev {
                } else {
                    debug!("Notification #{} = {:?}", i, &event);
                }

                if let Packet::Publish(p) = ev {
                    debug!("Publish #{} = {:?}", i, p);
                    let mut topic = p.topic.as_str();
                    if let Some(i) = topic.find('/') {
                        topic = &topic[i + 1..];
                    }
                    let msg = String::from_utf8_lossy(&p.payload);
                    debug!("Payload #{} = {} -- {}", i, topic, msg);
                    let json: Value = serde_json::from_str(&msg).unwrap_or_else(|_| json!({}));
                    debug!("Json #{} = {} -- {:?}", i, topic, json);
                    for k in [
                        "battery",
                        "humidity",
                        "linkquality",
                        "pressure",
                        "state",
                        "temperature",
                        "voltage",
                    ] {
                        if let Some(v) = json.get(k) {
                            let key = format!("{}/{}", topic, k);
                            if let Some(f) = v.as_f64() {
                                send_value(&opts.coap_url, key, f);
                            } else if let Some(s) = v.as_str() {
                                let f: f64 = match s.to_ascii_lowercase().as_str() {
                                    "on" => 1.0,
                                    "off" => 0.0,
                                    _ => -1.0,
                                };
                                send_value(&opts.coap_url, key, f);
                            }
                        }
                    }
                }
            }
            Event::Outgoing(_) => {}
        }
    }
    Ok(())
}

fn send_value<S1: AsRef<str>, S2: AsRef<str>>(url: S1, key: S2, value: f64) {
    let coap_payload = format!("{} {:.2}", key.as_ref(), value);
    info!("*** SEND {} <-- {}", url.as_ref(), coap_payload);
    let coap_result =
        CoAPClient::post_with_timeout(url.as_ref(), coap_payload.into_bytes(), Duration::new(2, 0));
    if let Err(e) = coap_result {
        error!("CoAP error: {:?}", e);
    }
}
// EOF

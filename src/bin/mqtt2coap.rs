// main.rs

// use anyhow::anyhow;
use coap::CoAPClient;
use log::*;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use serde_json::json;
use serde_json::Value;
use std::{fmt::Display, time::Duration};
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
        client.subscribe(format!("zigbee2mqtt/{topic}"), QoS::AtMostOnce)?;
    }

    // Iterate to poll the eventloop for connection progress
    for (i, notification) in connection.iter().enumerate() {
        let event = notification?;
        if let Event::Incoming(ev) = &event {
            if let Packet::PingResp = *ev {
                // Sigh, we don't have if not let...
            } else {
                // Debug output all events except ping response
                debug!("Notification #{i} = {event:?}");
            }

            if let Packet::Publish(p) = ev {
                // Whoa, we actually have a message to process
                debug!("Publish #{i} = {p:?}");
                let mut topic = p.topic.as_str();
                if let Some(i) = topic.find('/') {
                    // ignore all chars from topic until first '/' if there is one
                    topic = &topic[i + 1..];
                }
                let msg = String::from_utf8_lossy(&p.payload);
                debug!("Payload #{i} = {topic} -- {msg}");
                handle_msg(&opts, topic, msg);
            }
        }
    }
    Ok(())
}

fn handle_msg<S1, S2>(opts: &OptsCommon, topic: S1, msg: S2)
where
    S1: AsRef<str> + Display,
    S2: AsRef<str> + Display,
{
    let json: Value = serde_json::from_str(msg.as_ref()).unwrap_or_else(|_| json!({}));
    debug!("Json = {topic} -- {json:?}");
    for (k, v) in json.as_object().unwrap() {
        info!("JSON {k:?} = {v:?}");
        let key = format!("{topic}/{k}");
        let mut f: f64 = 0.0;
        let mut can_send = false;
        if v.is_f64() {
            f = v.as_f64().unwrap();
            can_send = true;
        } else if v.is_i64() {
            f = v.as_i64().unwrap() as f64;
            can_send = true;
        } else if v.is_u64() {
            f = v.as_u64().unwrap() as f64;
            can_send = true;
        } else if v.is_boolean() {
            f = 0.0;
            if v.as_bool().unwrap() {
                f = 1.0;
            }
            can_send = true;
        } else if v.is_string() {
            let s = v.as_str().unwrap();
            f = match s.to_ascii_lowercase().as_str() {
                "on" | "1" | "true" => 1.0,
                _ => 0.0,
            };
            can_send = true;
        }
        if can_send {
            send_value(opts, &key, f);
        }
    }
}

fn send_value<S>(opts: &OptsCommon, key: S, value: f64)
where
    S: AsRef<str> + Display,
{
    let payload = format!("{key} {value:.2}");
    let url = &opts.coap_url;
    info!("*** SEND {url} <-- {payload}");
    let coap_result = CoAPClient::post_with_timeout(url, payload.into_bytes(), Duration::new(2, 0));
    if let Err(e) = coap_result {
        error!("CoAP error: {e:?}");
    }
}
// EOF

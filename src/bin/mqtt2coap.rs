// main.rs

// use anyhow::anyhow;
use coap::CoAPClient;
use log::*;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use serde_json::json;
use serde_json::Value;
use std::{fmt::Display, time::Duration};
use structopt::StructOpt;

use mqtt2coap::*;

fn main() -> anyhow::Result<()> {
    let mut opts = OptsCommon::from_args();
    opts.finish()?;
    start_pgm(&opts, "mqtt2coap");
    debug!("Runtime config:\n{opts:#?}");

    let runtime = tokio::runtime::Runtime::new()?;

    let mut mqttoptions = MqttOptions::new("mqtt2coap", &opts.mqtt_host, opts.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 42);
    runtime.block_on(async move {
        run_mqtt(
            client,
            eventloop,
            opts.topics.to_owned(),
            opts.coap_url.to_owned(),
        )
        .await
    })
}

async fn run_mqtt(
    client: AsyncClient,
    mut eventloop: EventLoop,
    topics: String,
    coap_url: String,
) -> anyhow::Result<()> {
    for topic in topics.split(',') {
        let topic = format!("zigbee2mqtt/{topic}");
        debug!("Subscribing topic {topic}");
        client.subscribe(&topic, QoS::AtLeastOnce).await?;
    }

    // Iterate to poll the eventloop for connection progress
    let i: usize = 0;
    loop {
        let event = eventloop.poll().await?;
        debug!("mqtt event: {event:?}");
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
                handle_msg(&coap_url, topic, msg);
            }
        }
    }
}

fn handle_msg<S1, S2, S3>(coap_url: S1, topic: S2, msg: S3)
where
    S1: AsRef<str> + Display,
    S2: AsRef<str> + Display,
    S3: AsRef<str> + Display,
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
            coap_send(coap_url.as_ref(), &key, f);
        }
    }
}

fn coap_send<S1, S2>(url: S1, key: S2, value: f64)
where
    S1: AsRef<str> + Display,
    S2: AsRef<str> + Display,
{
    let payload = format!("{key} {value:.2}");
    info!("*** SEND {url} <-- {payload}");
    let coap_result =
        CoAPClient::post_with_timeout(url.as_ref(), payload.into_bytes(), Duration::new(2, 0));
    if let Err(e) = coap_result {
        error!("CoAP error: {e:?}");
    }
}
// EOF

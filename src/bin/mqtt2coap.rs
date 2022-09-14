// main.rs

use anyhow::*;
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
    opts.start_pgm(env!("CARGO_BIN_NAME"));
    debug!("Runtime config:\n{opts:#?}");

    let runtime = tokio::runtime::Runtime::new()?;

    let mut mqttoptions = MqttOptions::new("mqtt2coap", &opts.mqtt_host, opts.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(25));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 42);
    runtime.block_on(async move { run_mqtt(&opts, client, eventloop).await })
}

async fn run_mqtt(
    opts: &OptsCommon,
    client: AsyncClient,
    mut eventloop: EventLoop,
) -> anyhow::Result<()> {
    let prefix = &opts.topic_prefix;

    for topic in opts.topics.split(',') {
        let s_topic = format!("{prefix}{topic}");
        info!("Subscribing topic {s_topic}");
        client.subscribe(&s_topic, QoS::AtLeastOnce).await?;
    }

    // Iterate to poll the eventloop for connection progress
    let mut i: usize = 0;
    loop {
        i += 1;
        let event = eventloop.poll().await?;
        trace!("mqtt event: {event:?}");
        if let Event::Incoming(ev) = &event {
            match ev {
                Packet::PingResp => {
                    // silent
                }
                _ => {
                    // Debug output all other events
                    debug!("Notification #{i} = {event:?}");
                }
            }

            if let Packet::Publish(p) = ev {
                // Whoa, we actually have a message to process
                info!("Publish #{i} = {p:?}");
                let mut topic = p.topic.as_str();
                if let Some((_pre, post)) = topic.split_once('/') {
                    topic = post;
                }
                let msg = String::from_utf8_lossy(&p.payload);
                debug!("Payload #{i} = {topic} -- {msg}");
                if let Err(e) = handle_msg(&opts.coap_url, topic, msg) {
                    error!("Message handling error: {e}");
                }
            }
        }
    }
}

fn handle_msg<S1, S2, S3>(coap_url: S1, topic: S2, msg: S3) -> anyhow::Result<()>
where
    S1: AsRef<str> + Display,
    S2: AsRef<str> + Display,
    S3: AsRef<str> + Display,
{
    let json: Value = serde_json::from_str(msg.as_ref()).unwrap_or_else(|_| json!({}));
    debug!("Json = {topic} -- {json:?}");
    for (k, v) in json.as_object().ok_or_else(|| anyhow!("json error"))? {
        debug!("JSON {k:?} = {v:?}");
        let key = format!("{topic}/{k}");
        let mut f = 0.0;
        let mut can_send = false;
        if v.is_f64() {
            f = v.as_f64().unwrap_or_default();
            can_send = true;
        } else if v.is_i64() {
            f = v.as_i64().unwrap_or_default() as f64;
            can_send = true;
        } else if v.is_u64() {
            f = v.as_u64().unwrap_or_default() as f64;
            can_send = true;
        } else if v.is_boolean() {
            f = if v.as_bool().unwrap_or_default() {
                1.0
            } else {
                0.0
            };
            can_send = true;
        } else if v.is_string() {
            let s = v.as_str().unwrap_or_default();
            f = match s.to_ascii_lowercase().as_str() {
                "on" | "1" | "true" => 1.0,
                _ => 0.0,
            };
            can_send = true;
        }
        if can_send {
            coap_send(coap_url.as_ref(), &key, f)?;
        } else {
            error!("Could not parse json value: {v:?}");
        }
    }
    Ok(())
}

fn coap_send<S1, S2>(url: S1, key: S2, value: f64) -> anyhow::Result<()>
where
    S1: AsRef<str> + Display,
    S2: AsRef<str> + Display,
{
    let payload = format!("{key} {value:.2}");
    info!("*** CoAP POST {url} <-- {payload}");

    let res =
        CoAPClient::post_with_timeout(url.as_ref(), payload.into_bytes(), Duration::new(2, 0))?;
    info!("<-- {res:?}");
    Ok(())
}
// EOF

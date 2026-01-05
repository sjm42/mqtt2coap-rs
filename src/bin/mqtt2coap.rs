// bin/mqtt2coap.rs

use coap::UdpCoAPClient;
use rumqttc::{Event, MqttOptions, Packet, QoS};
use serde_json::{json, Value};

use mqtt2coap::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let me = env!("CARGO_BIN_NAME");
    let opts = OptsCommon::new(me);
    debug!("Runtime config:\n{opts:#?}");

    let mut mqttoptions = MqttOptions::new(me, &opts.mqtt_host, opts.mqtt_port);
    mqttoptions
        .set_keep_alive(Duration::from_secs(25))
        .set_clean_session(true);

    let (client, mut eventloop) = rumqttc::AsyncClient::new(mqttoptions, 42);
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

        let event = match eventloop.poll().await {
            Err(e) => {
                error!("MQTT event error: {e}");
                continue;
            }
            Ok(ev) => ev,
        };

        trace!("mqtt event: {event:?}");
        if let Event::Incoming(ev) = &event {
            match ev {
                Packet::PingResp => {
                    // silent
                }
                Packet::Publish(p) => {
                    // Whoa, we actually have a message to process
                    info!("Publish #{i} = {p:?}");

                    let mut topic = p.topic.as_str();
                    if let Some((_pre, post)) = topic.split_once('/') {
                        // strip prefix if found
                        topic = post;
                    }

                    let url = opts.coap_url.clone();
                    let topic = topic.to_string();
                    let msg = String::from_utf8_lossy(&p.payload).to_string();
                    debug!("Payload #{i} = {topic} -- {msg}");

                    tokio::spawn(handle_msg(url, topic, msg, i));
                }
                _ => {
                    // Debug output all other events
                    debug!("Notification #{i} = {event:?}");
                }
            }
        }
    }
}

async fn handle_msg(coap_url: String, topic: String, msg: String, i: usize) -> anyhow::Result<()> {
    let json: Value = serde_json::from_str(&msg).unwrap_or_else(|_| json!({}));
    debug!("Json = {topic} -- {json:?}");
    for (k, v) in json.as_object().ok_or_else(|| anyhow!("json error"))? {
        debug!("JSON {k:?} = {v:?}");
        let key = format!("{topic}/{k}");

        let f = if let Some(x) = v.as_f64() {
            x
        } else if let Some(b) = v.as_bool() {
            if b { 1.0 } else { 0.0 }
        } else if let Some(s) = v.as_str() {
            match s.to_ascii_lowercase().as_str() {
                "on" | "1" | "true" => 1.0,
                _ => 0.0,
            }
        } else {
            error!("Could not parse json value: {v:?}");
            continue;
        };
        if let Err(e) = coap_send(&coap_url, key.as_str(), f, i).await {
            error!("CoAP send error: #{i} {e}");
        }
    }
    Ok(())
}

async fn coap_send(url: &str, key: &str, value: f64, i: usize) -> anyhow::Result<()> {
    let payload = format!("{key} {value:.2}");
    info!("*** #{i} CoAP POST {url} <-- {payload}");

    let res = UdpCoAPClient::post_with_timeout(url, payload.into_bytes(), Duration::new(30, 0)).await?;
    info!("<-- {res:?}");
    Ok(())
}
// EOF

use crate::responses::response_list::ResponseList;
use crate::streams_subscriber::subscriber::Channel;

use actix_web::{web, Error, HttpRequest, HttpResponse};

use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn decode_channel(
    req: HttpRequest,
    node: web::Data<String>,
) -> Result<HttpResponse, Error> {
    let channel_root = req.match_info().get("channel_root");

    println!(
        "POST /decode_channel -- {:?}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    match channel_root {
        Some(data) => {
            let str_iter = data.split(":").collect::<Vec<&str>>();
            if str_iter.len() != 2 {
                return Ok(HttpResponse::Ok().json(ResponseList {
                    status: "Error: Invalid Address".to_string(),
                    messages: vec![],
                }));
            }
            let address = str_iter[0];
            let msg_id = str_iter[1];
            let mut subscriber: Channel = Channel::new(
                node.clone().to_string(),
                address.to_string(),
                msg_id.to_string(),
                None,
            );

            match subscriber.connect() {
                Ok(_) => {
                    let msg_list = match read_all_public(&mut subscriber).await {
                        Some(list) => Some(list),
                        None => {
                            return Ok(HttpResponse::Ok().json(ResponseList {
                                status: "Error: Could connect To Tangle".to_string(),
                                messages: vec![],
                            }));
                        }
                    };

                    if msg_list.is_none() {
                        return Ok(HttpResponse::Ok().json(ResponseList {
                            status: "Error: Could not Find Messages".to_string(),
                            messages: vec![],
                        }));
                    }

                    return Ok(HttpResponse::Ok().json(ResponseList {
                        status: "Success".to_string(),
                        messages: msg_list.unwrap(),
                    }));
                }
                Err(e) => {
                    println!("ERR");
                    return Ok(HttpResponse::Ok().json(ResponseList {
                        status: format!("Error: {}", e.to_string()),
                        messages: vec![],
                    }));
                }
            };
        }
        None => Ok(HttpResponse::Ok().json(format!("No thing!"))),
    }
}

async fn read_all_public(subscriber: &mut Channel) -> Option<Vec<String>> {
    let tag_list = match subscriber.get_next_message() {
        Some(v) => v,
        None => {
            return None;
        }
    };

    let mut msg_list: Vec<String> = vec![];
    for signed_message_tag in tag_list {
        let signed_packet_tag: String = signed_message_tag;
        let msgs: Vec<(Option<String>, Option<String>)> =
            match subscriber.read_signed(signed_packet_tag) {
                Ok(m) => m,
                Err(_) => return None,
            };
        for (msg_p, _msg_m) in msgs {
            match msg_p {
                None => continue,
                Some(message) => msg_list.push(message),
            }
        }
    }
    Some(msg_list)
}

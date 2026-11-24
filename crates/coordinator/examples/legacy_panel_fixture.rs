use std::{
    io::{self, Read, Write},
    net::TcpListener,
    time::Duration,
};

fn inbound_details() -> String {
    use serde_json::json;
    let settings = json!({"clients":[{
        "id":"11111111-1111-4111-8111-111111111111", "email":"fixture-client",
        "password":"fixture-worker-password", "security":"auto", "flow":"",
        "enable":true, "expiryTime":1893456001000_i64, "totalGB":0, "limitIp":0,
        "reset":0, "tgId":"", "subId":"fixtureSubscription", "comment":""
    },{
        "email":"fixture-ip-limited", "enable":true, "limitIp":1
    },{
        "email":"fixture-capped", "enable":true
    },{
        "email":"fixture-disabled-traffic", "enable":true
    },{
        "enable":false
    }]});
    // The original Garlic worker reader expects settings as an object.
    // The standard panel API returns the same value serialized as a string.
    let settings = if std::env::var("GARLIC_FIXTURE_WORKER_OBJECT").as_deref() == Ok("1") {
        settings
    } else {
        json!(settings.to_string())
    };
    json!({"success":true,"msg":"","obj":{
        "id":22,"userId":1,"nodeId":7,"enable":true,"remark":"Remote Trojan",
        "protocol":"trojan","port":443,"listen":"127.0.0.1","tag":"fixture-inbound",
        "up":5,"down":7,"total":0,"allTime":12,"expiryTime":0,"trafficReset":"never",
        "lastTrafficResetTime":0,"streamSettings":"{}","sniffing":"{}",
        "settings":settings,
        "clientStats":[{"id":1,"inboundId":22,"email":"fixture-client","up":5,"down":7,
            "total":0,"allTime":12,"enable":true,"expiryTime":1893456000000_i64,
            "reset":0,"lastOnline":0},
            {"email":"fixture-ip-limited","enable":true,"total":0},
            {"email":"fixture-capped","enable":true,"total":1024},
            {"email":"fixture-disabled-traffic","enable":false,"total":0}]
    }})
    .to_string()
}

fn serve(mut peer: impl Read + Write) -> io::Result<()> {
    let mut input = Vec::new();
    let mut buffer = [0; 2048];
    let (header_end, body_size) = loop {
        let received = peer.read(&mut buffer)?;
        if received == 0 || input.len() > 65_536 {
            return Err(io::Error::other("missing complete fixture request"));
        }
        input.extend_from_slice(&buffer[..received]);
        if let Some(end) = input.windows(4).position(|b| b == b"\r\n\r\n") {
            let header = String::from_utf8_lossy(&input[..end]);
            let len = header
                .lines()
                .filter_map(|line| line.split_once(':'))
                .find(|(key, _)| key.eq_ignore_ascii_case("Content-Length"))
                .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                .unwrap_or(0);
            if len > 65_536 {
                return Err(io::Error::other("fixture request body is too large"));
            }
            break (end + 4, len);
        }
    };
    while input.len() < header_end + body_size {
        let received = peer.read(&mut buffer)?;
        if received == 0 {
            return Err(io::Error::other("missing fixture body"));
        }
        input.extend_from_slice(&buffer[..received]);
    }
    let header = String::from_utf8_lossy(&input[..header_end]);
    let first = header.lines().next().unwrap_or_default();
    let target = first.split_ascii_whitespace().nth(1).unwrap_or("/");
    let method = first.split_ascii_whitespace().next().unwrap_or("");
    let valid = header.lines().any(|line| {
        line.split_once(':').is_some_and(|(key, value)| {
            key.eq_ignore_ascii_case("authorization")
                && value.trim() == "Bearer local-fixture-panel-token"
        })
    });
    let body = String::from_utf8_lossy(&input[header_end..header_end + body_size]);
    let creates_client = target.ends_with("/clients/add");
    let create_contract = if creates_client {
        let request: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        let client = &request["client"];
        let matches_secret = |field: &str| {
            client[field].as_str().is_some_and(|value| {
                value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
        };
        matches_secret("password")
            && matches_secret("subId")
            && client["email"] == "fixture-created"
            && client["security"] == "auto"
            && client["enable"] == true
            && request["inboundIds"] == serde_json::json!([22])
    } else {
        true
    };
    if creates_client {
        println!("{method} {target} valid-auth={valid} create-contract={create_contract}");
    } else {
        println!("{method} {target} valid-auth={valid} {body}");
    }
    let detail;
    let data = if !valid {
        r#"{"success":false,"msg":"missing fixture token","obj":null}"#
    } else if target.ends_with("/inbounds/options") {
        r#"{"success":true,"msg":"","obj":[{"id":11,"enable":true,"remark":"Local Trojan","protocol":"trojan","port":443},{"id":22,"nodeId":7,"enable":true,"remark":"Remote Trojan","protocol":"trojan","port":443},{"id":33,"enable":true,"remark":"Local VLESS","protocol":"vless","port":443},{"id":44,"nodeId":7,"enable":true,"remark":"Remote VLESS","protocol":"vless","port":443},{"id":99,"nodeId":9,"enable":true,"remark":"Unrelated fixture","protocol":"trojan","port":443}]}"#
    } else if target.ends_with("/inbounds/get/22") {
        detail = inbound_details();
        &detail
    } else if target.ends_with("/inbounds/list") {
        r#"{"success":true,"msg":"","obj":[{"id":11,"nodeId":0,"enable":true,"remark":"Local fixture","protocol":"vless","port":443,"settings":"{\"clients\":[{\"id\":\"11111111-1111-4111-8111-111111111111\",\"email\":\"fixture-client\",\"enable\":true,\"expiryTime\":1893456000000,\"totalGB\":2147483648,\"subId\":\"fixtureSubscription\"}]}","clientStats":[{"email":"fixture-client","up":11,"down":29,"total":2147483648,"enable":true,"expiryTime":1893456000000}]},{"id":22,"nodeId":7,"enable":true,"remark":"Remote fixture","protocol":"vless","port":443,"settings":"{\"clients\":[{\"id\":\"11111111-1111-4111-8111-111111111111\",\"email\":\"fixture-client\",\"enable\":true,\"expiryTime\":1893456001000,\"totalGB\":2147483648,\"subId\":\"fixtureSubscription\"}]}","clientStats":[{"email":"fixture-client","up":5,"down":7,"total":2147483648,"enable":true,"expiryTime":1893456001000}]}]}"#
    } else if target.ends_with("/clients/list") {
        r#"{"success":true,"msg":"","obj":[{"email":"fixture-client","id":"11111111-1111-4111-8111-111111111111","enable":true,"expiryTime":1893456000000,"totalGB":2147483648,"subId":"fixtureSubscription"}]}"#
    } else if target.contains("/clients/list/paged?") {
        r#"{"success":true,"msg":"","obj":{"items":[{"email":"fixture-client","enable":true,"expiryTime":1893456000000,"totalGB":2147483648,"inboundIds":[11,22,99],"traffic":{"up":81,"down":44,"total":2147483648},"subId":"fixtureSubscription"},{"email":"fixture-other","enable":true,"expiryTime":0,"totalGB":1024,"inboundIds":[99]}],"page":1,"pageSize":100,"total":2,"filtered":2}}"#
    } else if target.contains("/clients/get/") {
        r#"{"success":true,"msg":"","obj":{"client":{"email":"fixture-client","subId":"fixtureSubscription","enable":true,"expiryTime":1893456000000},"inboundIds":[11,22]}}"#
    } else if creates_client && create_contract {
        r#"{"success":true,"msg":"","obj":null}"#
    } else if target.ends_with("/clients/bulkAdjust") {
        r#"{"success":true,"msg":"","obj":{"adjusted":1}}"#
    } else {
        r#"{"success":false,"msg":"unhandled fixture route","obj":null}"#
    };
    write!(peer, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", data.len(), data)?;
    peer.flush()
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 18723))?;
    println!("Local panel fixture listening");
    for peer in listener.incoming().flatten() {
        peer.set_read_timeout(Some(Duration::from_secs(15)))?;
        if let Err(error) = serve(peer) {
            println!("Fixture transport: {:?}", error.kind());
        }
    }
    Ok(())
}

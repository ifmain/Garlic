use garlic_coordinator::{
    accounts::{
        admin::Manage,
        operator::{self, Change},
        AccountsService,
    },
    devices::DeviceStore,
    files,
};
use garlic_relay_data::control::{Authority, Role};
use serde_json::{json, Value};
use uuid::Uuid;

fn manage(
    s: &AccountsService,
    token: &str,
    body: Value,
) -> Result<Value, garlic_relay_session::SessionError> {
    s.admin_manage(token, serde_json::from_value::<Manage>(body)?)
}
#[test]
fn application_keys_follow_account_renames_roles_and_subscription() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    operator::initialize(&path).unwrap();
    operator::add(&path, "operator", "password-fixture-only").unwrap();
    operator::change(&path, "operator", Change::Role(Role::Admin)).unwrap();
    let owner = Authority {
        owner_id: Uuid::new_v4(),
        epoch: 1,
    };
    let service = AccountsService::new(path.clone(), dir.path().join("keys.json"), owner);
    let admin = service
        .login("operator", "password-fixture-only")
        .unwrap()
        .token;
    let created = manage(
        &service,
        &admin,
        json!({"action":"create","username":"fixture"}),
    )
    .unwrap();
    let key = created["token"].as_str().unwrap();
    let (original, session) = service.user(key).unwrap();
    assert_eq!(session.expires_at - session.issued_at, 31_536_000);
    assert!(session.owner.owner_id.is_nil());
    assert_eq!(session.owner.epoch, 0);
    let snapshot = service.snapshot().unwrap();
    assert!(snapshot
        .portable_sessions
        .contains_key(&garlic_coordinator::accounts::token_hash(key)));
    assert!(!snapshot
        .portable_sessions
        .contains_key(&garlic_coordinator::accounts::token_hash(&admin)));
    assert!(key.starts_with(&format!("gu_{}_", original.id.simple())));
    assert_eq!(key.len(), 68);
    assert!(service.admin_users(key).is_err());
    assert!(manage(
        &service,
        key,
        json!({"action":"role","username":"fixture","role":"admin"})
    )
    .is_err());
    let second = manage(
        &service,
        &admin,
        json!({"action":"issue_key","username":"fixture"}),
    )
    .unwrap();
    let second_key = second["token"].as_str().unwrap();
    assert_ne!(key, second_key);
    assert!(!std::fs::read_to_string(&path).unwrap().contains(key));
    manage(
        &service,
        &admin,
        json!({"action":"rename","username":"fixture","new_username":"renamed"}),
    )
    .unwrap();
    for bearer in [key, second_key] {
        let (u, _) = service.user(bearer).unwrap();
        assert_eq!(u.username, "renamed");
        assert_eq!(u.generation, original.generation);
    }
    manage(
        &service,
        &admin,
        json!({"action":"disable","username":"renamed"}),
    )
    .unwrap();
    assert!(service.user(key).is_err());
    manage(
        &service,
        &admin,
        json!({"action":"enable","username":"renamed"}),
    )
    .unwrap();
    assert!(service.user(second_key).is_err());
    assert!(manage(
        &service,
        &admin,
        json!({"action":"create","username":"expired","subscription":"1"})
    )
    .is_err());
    let list = service.admin_users(&admin).unwrap();
    assert!(list
        .iter()
        .any(|u| u["username"] == "expired" && u["status"] == "expired"));
    assert!(!serde_json::to_string(&list)
        .unwrap()
        .contains("password_hash"));
}

#[test]
fn simultaneous_device_claims_preserve_generation_and_unknown_state() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("auth.json");
    let uid = Uuid::new_v4();
    let leases = DeviceStore::for_accounts(&file);
    let winners: Vec<_> = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| leases.claim(uid, &Uuid::new_v4().to_string(), false)))
            .collect();
        threads
            .into_iter()
            .filter_map(|t| t.join().unwrap().ok())
            .collect()
    });
    assert_eq!(winners.len(), 1);
    let old = &winners[0];
    let device_file = file.with_extension("devices.json");
    files::edit(&device_file, |raw: &mut Value| {
        raw["future_field"] = json!(42);
        raw["leases"][uid.to_string()]["keep"] = json!(true);
        Ok(())
    })
    .unwrap();
    let takeover = leases
        .claim(uid, &Uuid::new_v4().to_string(), true)
        .unwrap();
    assert_eq!(takeover.generation, old.generation + 1);
    assert!(leases.valid(uid, &old.token).is_err());
    assert!(leases.update(uid, &old.token, true).is_err());
    leases.valid(uid, &takeover.token).unwrap();
    leases.update(uid, &takeover.token, true).unwrap();
    assert!(leases.valid(uid, &takeover.token).is_err());
    let raw: Value = files::read(&device_file).unwrap();
    assert_eq!(raw["future_field"], 42);
    assert_eq!(raw["leases"][uid.to_string()]["keep"], true);
    let next = leases
        .claim(uid, &Uuid::new_v4().to_string(), false)
        .unwrap();
    assert_eq!(next.generation, takeover.generation + 1);
}

//! Offline administration of Garlic account files and provisioned node identities.
use garlic_coordinator::accounts::admin::parse_subscription;
use garlic_coordinator::{
    accounts::{
        now,
        operator::{self, Change},
    },
    files,
};
use garlic_relay_data::control::Role;
use garlic_relay_session::{
    identity::{Challenge, Identity, IdentityFile, KeyRing},
    SessionError,
};
use std::{
    io::{self, BufRead},
    path::Path,
};

fn password() -> Result<String, SessionError> {
    let mut value = String::new();
    io::stdin().lock().read_line(&mut value)?;
    while value.ends_with(['\r', '\n']) {
        value.pop();
    }
    Ok(value)
}
fn cli(args: &[&str]) -> Result<(), SessionError> {
    match args {
        ["keys", "init", p, id] => files::create(Path::new(p), &KeyRing::create(id)?)?,
        ["keys", "status", p] => println!("{}", files::read::<KeyRing>(Path::new(p))?.status()),
        ["keys", op, p, id] => files::edit(Path::new(p), |ring: &mut KeyRing| match *op {
            "add" => ring.add(id),
            "activate" => ring.activate(id),
            "retire" => ring.retire(id),
            _ => Err("unknown key operation".into()),
        })?,
        ["users", "init", p] => operator::initialize(Path::new(p))?,
        ["users", "list", p] => {
            println!("{}", serde_json::to_string(&operator::list(Path::new(p))?)?)
        }
        ["users", "add", p, name, "--password-stdin"] | ["users", "add", p, name] => {
            let user = operator::add(Path::new(p), name, &password()?)?;
            println!("user_id={}", user.id);
        }
        ["users", "password", p, name, "--password-stdin"] | ["users", "password", p, name] => {
            operator::change(Path::new(p), name, Change::Password(password()?))?
        }
        ["users", "role", p, name, role] => operator::change(
            Path::new(p),
            name,
            Change::Role(match *role {
                "admin" => Role::Admin,
                "client" => Role::Client,
                _ => return Err("invalid role".into()),
            }),
        )?,
        ["users", "subscription", p, name, sub] => operator::change(
            Path::new(p),
            name,
            Change::Subscription(parse_subscription(sub)?),
        )?,
        ["users", op, p, name] => operator::change(
            Path::new(p),
            name,
            match *op {
                "disable" => Change::Enabled(false),
                "enable" => Change::Enabled(true),
                "remove" => Change::Remove,
                _ => return Err("unknown user operation".into()),
            },
        )?,
        ["nodes", "list", p] => println!(
            "{}",
            serde_json::to_string(&operator::nodes(Path::new(p))?)?
        ),
        ["nodes", "revoke", p, id] => operator::revoke(Path::new(p), id)?,
        ["identity", "challenge", p, k, label] => {
            let ring = files::read(Path::new(k))?;
            let request = files::edit_or_create(Path::new(p), IdentityFile::generate, |file| {
                Identity::load(file)?.request(&ring, label, now())
            })?;
            println!("{}", serde_json::to_string(&request)?);
        }
        ["identity", "prove", p, k] => {
            let node = Identity::load(&files::read(Path::new(p))?)?;
            let ring = files::read(Path::new(k))?;
            let challenge: Challenge = serde_json::from_reader(io::stdin().lock())?;
            challenge.request.verify(&ring, now())?;
            println!(
                "{}",
                serde_json::to_string(&node.prove(
                    &ring,
                    &challenge.request,
                    &challenge,
                    now()
                )?)?
            );
        }
        _ => {
            return Err(
                "usage: garlic-auth keys|users|nodes|identity OPERATION FILE [arguments]".into(),
            )
        }
    }
    Ok(())
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Err(e) = cli(&args.iter().map(String::as_str).collect::<Vec<_>>()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

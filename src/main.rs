use indicatif::ProgressBar;
use serde::Deserialize;
use std::{env, thread, time::Duration};

#[derive(Debug, Deserialize)]
struct UserBasic {
    id: u64,
    #[serde(rename = "active?")]
    active: bool,
}

#[derive(Debug, Deserialize)]
struct UserDetail {
    login: String,
    cursus_users: Vec<CursusUser>,
}

#[derive(Debug, Deserialize)]
struct CursusUser {
    level: f64,
    cursus: Cursus,
}

#[derive(Debug, Deserialize)]
struct Cursus {
    slug: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth_header = format!(
        "Bearer {}",
        env::var("TOKEN").expect("the bearer token to be in $TOKEN")
    );

    // 1) Get the list of users for the given campus and pool filters
    let url = "https://api.intra.42.fr/v2/campus/1/users?filter[pool_year]=2022&filter[pool_month]=july,august,september";

    // Parse the JSON array into a Vec<UserBasic>
    let users: Vec<UserBasic> = ureq::get(url)
        .header("Authorization", &auth_header)
        .call()?
        .body_mut()
        .read_json()?;

    // 2) Filter only active users
    let active_users: Vec<UserBasic> = users.into_iter().filter(|u| u.active).collect();

    println!("found {} active users", active_users.len());

    let p = ProgressBar::new(active_users.len() as u64);
    let mut results = Vec::new();

    // 3) For each active user, fetch their profile and find the 42cursus level
    for user in active_users {
        // Sleep to respect the 2 requests/s rate limit
        thread::sleep(Duration::from_millis(500));

        let user_url = format!("https://api.intra.42.fr/v2/users/{}", user.id);
        let detail: UserDetail = ureq::get(&user_url)
            .header("Authorization", &auth_header)
            .call()?
            .body_mut()
            .read_json()?;

        // Find the cursus where `slug` is "42cursus"
        let level_42cursus = detail
            .cursus_users
            .iter()
            .find(|cu| cu.cursus.slug == "42cursus")
            .map(|cu| cu.level);

        if let Some(level) = level_42cursus {
            results.push((detail.login, level));
        }
        p.inc(1);
    }

    // 4) Sort descending by level
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // 5) Print the first 20
    println!("Top (by 42cursus level):");
    for (i, (login, level)) in results.iter().enumerate() {
        println!("{}. {} - level {}", i + 1, login, level);
    }

    Ok(())
}

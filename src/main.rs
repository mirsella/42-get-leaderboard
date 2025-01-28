use indicatif::ProgressBar;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashSet,
    env, thread,
    time::{Duration, Instant},
};

#[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
struct User {
    id: u64,
    login: String,
    #[serde(rename = "active?")]
    active: bool,

    // We don't necessarily get `cursus_users` in the short listing
    #[serde(default)]
    cursus_users: Vec<Value>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth_header = format!(
        "Bearer {}",
        env::var("TOKEN").expect("the bearer token to be in $TOKEN")
    );

    // 1) Get the list of users for the campus and pool filters
    let pages = 5;
    println!("getting first {pages} pages of /users");
    let p = ProgressBar::new(pages);
    let users: HashSet<User> = (1..=pages).flat_map(|i| {
        let url = format!("https://api.intra.42.fr/v2/campus/1/users?filter[pool_year]=2022&filter[pool_month]=july,august,september&per_page=100&page={i}");
        let s= ureq::get(url)
            .header("Authorization", &auth_header)
            .call().unwrap()
            .body_mut()
            .read_json::<HashSet<_>>().unwrap();
            p.inc(1);
        s
    }).collect();
    p.finish();

    // 2) Filter only active users
    print!("got {} students", users.len());
    let active_users: Vec<User> = users.into_iter().filter(|u| u.active).collect();
    println!(" ({} active)", active_users.len());

    // 3) For each active user, fetch detailed profile and find their 42cursus level
    let p = ProgressBar::new(active_users.len() as u64);
    let mut last = Instant::now() - Duration::from_millis(500);
    let mut results = active_users
        .iter()
        .flat_map(|user| {
            // Sleep ~500ms to respect 2 requests/s rate limit
            thread::sleep(Duration::from_millis(500).saturating_sub(Instant::now() - last));
            last = Instant::now();
            let user_url = format!("https://api.intra.42.fr/v2/users/{}", user.id);
            let detail: User = ureq::get(&user_url)
                .header("Authorization", &auth_header)
                .call()
                .unwrap()
                .body_mut()
                .read_json()
                .unwrap();
            p.inc(1);

            // Extract the "42cursus" level (if any) from `detail.cursus_users`
            // Each item in `cursus_users` looks like:
            // {
            //   "level": f64,
            //   "cursus": { "slug": "42cursus" },
            //   ...
            // }
            detail
                .cursus_users
                .iter()
                .find_map(|cu| {
                    let slug = cu
                        .get("cursus")
                        .and_then(|c| c.get("slug"))
                        .and_then(|s| s.as_str());

                    if slug == Some("42cursus") {
                        cu.get("level").and_then(|lv| lv.as_f64())
                    } else {
                        None
                    }
                })
                .map(|l| (detail.login, l))
        })
        .collect::<Vec<_>>();
    p.finish();

    // 4) Sort descending by level
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Print out the final list (or top 20 if desired)
    println!("Top (by 42cursus level):");
    for (i, (login, level)) in results.iter().enumerate() {
        println!("{}. {} - level {}", i + 1, login, level);
    }

    Ok(())
}
